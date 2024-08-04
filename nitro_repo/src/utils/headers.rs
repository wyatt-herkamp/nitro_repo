use axum::http::HeaderMap;
use http::{
    header::{AsHeaderName, ToStrError},
    HeaderValue,
};
use nr_core::utils::base64_utils;
use strum::Display;
use thiserror::Error;
use tracing::error;

use crate::error::internal_error::InternalError;

pub fn get_accept(header_map: &HeaderMap) -> Result<Option<String>, InternalError> {
    let Some(header_value) = header_map.get("accept") else {
        return Ok(None);
    };

    let accept = header_value
        .to_str()
        .map(|x| x.to_string())
        .inspect_err(|err| {
            error!("Failed to convert accept header to string: {}", err);
        })
        .ok();
    Ok(accept)
}
#[derive(Debug, Display)]
pub enum InvalidStringOrOtherError<T> {
    InvalidString(ToStrError),
    Other(T),
}
pub trait HeaderValueExt {
    fn to_string(&self) -> Result<String, ToStrError>;
    fn to_string_as_option(&self) -> Option<String>;
    fn parsed<T: TryFrom<String>>(&self) -> Result<T, InvalidStringOrOtherError<T::Error>>;
}
impl HeaderValueExt for HeaderValue {
    fn to_string(&self) -> Result<String, ToStrError> {
        self.to_str().map(|x| x.to_string())
    }

    fn to_string_as_option(&self) -> Option<String> {
        self.to_str()
            .map(|x| x.to_string())
            .inspect_err(|error| {
                error!("Failed to convert header value to string: {}", error);
            })
            .ok()
    }

    fn parsed<T: TryFrom<String>>(&self) -> Result<T, InvalidStringOrOtherError<T::Error>> {
        let value = self
            .to_string()
            .map_err(InvalidStringOrOtherError::InvalidString)?;
        T::try_from(value).map_err(InvalidStringOrOtherError::Other)
    }
}
#[derive(Debug, Error)]
pub enum InvalidAuthorizationHeader {
    #[error("Invalid Authorization  Schema")]
    InvalidScheme,
    #[error("Invalid Authorization Value")]
    InvalidValue,
    #[error("Invalid Authorization Format. Expected: (Schema Type) (Value)")]
    InvalidFormat,
}
#[derive(Debug)]
pub enum AuthorizationHeader {
    Basic { username: String, password: String },
    Bearer { token: String },
    Session { session: String },
    Other { scheme: String, value: String },
}
impl TryFrom<String> for AuthorizationHeader {
    type Error = InvalidAuthorizationHeader;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let parts: Vec<&str> = value.split(' ').collect();
        if parts.len() != 2 {
            return Err(InvalidAuthorizationHeader::InvalidFormat);
        }
        let scheme = parts[0];
        let value = parts[1];
        match scheme {
            "Basic" => parse_basic_header(value).map_err(|err| {
                error!("Failed to parse basic header: {}", err);
                InvalidAuthorizationHeader::InvalidValue
            }),
            "Bearer" => Ok(AuthorizationHeader::Bearer {
                token: value.to_owned(),
            }),
            "Session" => Ok(AuthorizationHeader::Session {
                session: value.to_owned(),
            }),
            _ => Ok(AuthorizationHeader::Other {
                scheme: scheme.to_owned(),
                value: value.to_owned(),
            }),
        }
    }
}
fn parse_basic_header(header: &str) -> Result<AuthorizationHeader, InvalidAuthorizationHeader> {
    let parts: Vec<&str> = header.split(' ').collect();
    if parts.len() != 2 {
        return Err(InvalidAuthorizationHeader::InvalidFormat);
    }
    let value = parts[1];
    let decoded = base64_utils::decode(value).map_err(|err| {
        error!("Failed to decode base64: {}", err);
        InvalidAuthorizationHeader::InvalidValue
    })?;
    let decoded = String::from_utf8(decoded).map_err(|err| {
        error!("Failed to convert bytes to string: {}", err);
        InvalidAuthorizationHeader::InvalidValue
    })?;
    let parts: Vec<&str> = decoded.split(':').collect();
    if parts.len() != 2 {
        return Err(InvalidAuthorizationHeader::InvalidValue);
    }
    let username = parts[0];
    let password = parts[1];
    Ok(AuthorizationHeader::Basic {
        username: username.to_owned(),
        password: password.to_owned(),
    })
}

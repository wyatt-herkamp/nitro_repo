use nr_core::utils::base64_utils;
use tracing::{error, instrument};

use crate::utils::bad_request::{BadRequestErrors, InvalidAuthorizationHeader};

#[derive(Debug)]
pub enum AuthorizationHeader {
    Basic { username: String, password: String },
    Bearer { token: String },
    Session { session: String },
    Other { scheme: String, value: String },
}
impl TryFrom<String> for AuthorizationHeader {
    type Error = BadRequestErrors;
    #[instrument(skip(value), name = "AuthorizationHeader::try_from")]
    fn try_from(value: String) -> Result<Self, Self::Error> {
        let parts: Vec<&str> = value.split(' ').collect();

        if parts.len() != 2 {
            return Err(BadRequestErrors::InvalidAuthorizationHeader(
                InvalidAuthorizationHeader::InvalidFormat,
            ));
        }
        let scheme = parts[0];
        let value = parts[1];
        match scheme {
            "Basic" => parse_basic_header(value),
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
#[instrument(skip(header))]
fn parse_basic_header(header: &str) -> Result<AuthorizationHeader, BadRequestErrors> {
    let decoded = base64_utils::decode(header).map_err(|err| {
        error!("Failed to decode base64: {}", err);
        InvalidAuthorizationHeader::InvalidValue
    })?;
    let decoded = String::from_utf8(decoded).map_err(|err| {
        error!("Failed to convert bytes to string: {}", err);
        InvalidAuthorizationHeader::InvalidValue
    })?;
    let parts: Vec<&str> = decoded.split(':').collect();
    if parts.len() != 2 {
        return Err(InvalidAuthorizationHeader::InvalidBasicValue.into());
    }
    let username = parts[0];
    let password = parts[1];
    Ok(AuthorizationHeader::Basic {
        username: username.to_owned(),
        password: password.to_owned(),
    })
}

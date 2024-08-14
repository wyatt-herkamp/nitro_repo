use http::{header::ToStrError, HeaderValue};
use nr_core::utils::base64_utils;
use tracing::{debug, error};

use crate::error::{BadRequestErrors, InvalidAuthorizationHeader};

pub trait HeaderValueExt {
    fn to_string(&self) -> Result<String, ToStrError>;
    fn to_string_as_option(&self) -> Option<String>;
    fn parsed<T: TryFrom<String, Error = BadRequestErrors>>(&self) -> Result<T, BadRequestErrors>;
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

    fn parsed<T: TryFrom<String, Error = BadRequestErrors>>(&self) -> Result<T, BadRequestErrors> {
        let value = self.to_string()?;
        T::try_from(value)
    }
}

#[derive(Debug)]
pub enum AuthorizationHeader {
    Basic { username: String, password: String },
    Bearer { token: String },
    Session { session: String },
    Other { scheme: String, value: String },
}
impl TryFrom<String> for AuthorizationHeader {
    type Error = BadRequestErrors;

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
pub mod date_time {
    use chrono::FixedOffset;
    use http::HeaderValue;

    use crate::error::BadRequestErrors;

    pub fn date_time_for_header(date_time: &chrono::DateTime<FixedOffset>) -> HeaderValue {
        let date_time = date_time.with_timezone(&chrono::Utc);
        let date_time = date_time.format("%a, %d %b %Y %H:%M:%S GMT").to_string();
        HeaderValue::from_str(date_time.as_str()).expect("Failed to convert date time to header")
    }
    pub fn parse_date_time(
        header_value: &HeaderValue,
    ) -> Result<chrono::DateTime<FixedOffset>, BadRequestErrors> {
        let date_time = header_value.to_str()?;
        chrono::DateTime::parse_from_rfc2822(date_time).map_err(BadRequestErrors::from)
    }
}

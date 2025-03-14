use http::{HeaderName, HeaderValue, header::ToStrError};
use tracing::{error, warn};
pub mod date_time;
/// Extension trait for [http::HeaderValue]
pub trait HeaderValueExt {
    /// Converts the header value to a string
    fn to_string(&self) -> Result<String, ToStrError>;
    /// Converts the header value to a string
    fn to_string_as_option(&self) -> Option<String>;
    /// Parses the header value into a type Over the [TryFrom] trait
    ///
    /// Error must be convertible from [ToStrError]
    fn parsed<T, E>(&self) -> Result<T, E>
    where
        T: TryFrom<String, Error = E>,
        E: From<ToStrError>;
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

    fn parsed<T, E>(&self) -> Result<T, E>
    where
        T: TryFrom<String, Error = E>,
        E: From<ToStrError>,
    {
        let value = self.to_string()?;
        T::try_from(value)
    }
}

pub trait HeaderMapExt {
    fn get_string_ignore_empty(&self, key: &HeaderName) -> Option<String>;
    fn get_str_ignore_empty<'headers>(&'headers self, key: &HeaderName) -> Option<&'headers str>;
}

impl HeaderMapExt for http::HeaderMap {
    fn get_string_ignore_empty(&self, header: &HeaderName) -> Option<String> {
        self.get(header)
            .and_then(|v| v.to_str().ok())
            .and_then(|v| {
                if v.is_empty() {
                    warn!(?header, "Empty header Value",);
                    None
                } else {
                    Some(v.to_owned())
                }
            })
    }

    fn get_str_ignore_empty<'headers>(&'headers self, key: &HeaderName) -> Option<&'headers str> {
        self.get(key).and_then(|v| v.to_str().ok()).and_then(|v| {
            if v.is_empty() {
                warn!(?key, "Empty header Value",);
                None
            } else {
                Some(v)
            }
        })
    }
}

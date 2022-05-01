use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub struct NPMError(String);

impl Display for NPMError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Error for NPMError {}

impl From<&str> for NPMError {
    fn from(err: &str) -> NPMError {
        NPMError(err.to_string())
    }
}

impl From<String> for NPMError {
    fn from(err: String) -> NPMError {
        NPMError(err)
    }
}

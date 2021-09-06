use std::error::Error;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

use crate::error::internal_error::InternalError;

pub mod internal_error;
pub mod request_error;
#[derive(Debug)]
pub struct GenericError {
    pub error: String,
}

impl Display for GenericError {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "Error! {}. ({:?})", self.error, self.error)
    }
}

impl Error for GenericError {
    fn description(&self) -> &str {
        self.error.as_str()
    }
}

impl From<String> for GenericError {
    fn from(value: String) -> Self {
        GenericError { error: value }
    }
}

impl From<&str> for GenericError {
    fn from(value: &str) -> Self {
        GenericError {
            error: value.to_string(),
        }
    }
}

impl FromStr for GenericError {
    type Err = InternalError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(GenericError {
            error: s.to_string(),
        })
    }
}

use std::error::Error;
use std::fmt::{Display, Formatter};
use actix_web::HttpResponse;
use crate::authentication::UnAuthorized;
use crate::system::permissions::options::MissingPermission;
use crate::system::permissions::PermissionError;
use thiserror::Error;
use crate::repository::error::RepositoryError;
use crate::storage::models::StorageError;

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
        NPMError(err.to_string())
    }
}


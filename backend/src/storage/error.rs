use std::time::SystemTimeError;
use this_actix_error::ActixError;

use thiserror::Error;

#[derive(Error, Debug, ActixError)]
pub enum StorageError {
    #[error("{0}")]
    LoadFailure(String),
    #[error("IO error {0}")]
    IOError(std::io::Error),
    #[error("JSON error {0}")]
    JSONError(serde_json::Error),
    #[error("Storage Already Exists!")]
    StorageAlreadyExist,
    #[error("Unable to create a new storages {0}")]
    StorageCreateError(String),
    #[error("Repository Already Exists")]
    #[status_code(CONFLICT)]
    RepositoryAlreadyExists,
    #[error("Missing Repository")]
    #[status_code(NOT_FOUND)]
    RepositoryMissing,
    #[error("Unable to find Parent Directory")]
    ParentIssue,
    #[error("Internal Error: {0}")]
    InternalError(String),
    #[error("Storage Delete Error: {0}")]
    StorageDeleteError(String),
}

impl From<std::io::Error> for StorageError {
    fn from(err: std::io::Error) -> StorageError {
        StorageError::IOError(err)
    }
}
impl From<SystemTimeError> for StorageError {
    fn from(err: SystemTimeError) -> StorageError {
        StorageError::InternalError(err.to_string())
    }
}

impl From<serde_json::Error> for StorageError {
    fn from(err: serde_json::Error) -> StorageError {
        StorageError::JSONError(err)
    }
}

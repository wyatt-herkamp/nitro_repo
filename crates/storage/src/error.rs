use std::time::SystemTimeError;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum StorageError {
    #[error("{0}")]
    LoadFailure(String),
    #[error("IO error {0}")]
    IOError(#[from] std::io::Error),
    #[error("JSON error {0}")]
    JSONError(#[from] serde_json::Error),
    #[error("Unable to create a new storages {0}")]
    StorageCreateError(String),
    #[error("Unable to find Parent Directory")]
    ParentIssue,
    #[error("Internal Error: {0}")]
    InternalError(String),
    #[error("Storage Delete Error: {0}")]
    StorageDeleteError(String),
    #[error("Invalid Config Type. Expected {0}")]
    InvalidConfigType(&'static str),
    #[error("Config Error: {0}")]
    ConfigError(&'static str),
}

impl From<SystemTimeError> for StorageError {
    fn from(err: SystemTimeError) -> StorageError {
        StorageError::InternalError(err.to_string())
    }
}

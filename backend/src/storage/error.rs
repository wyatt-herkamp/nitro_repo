use crate::api_response::APIResponse;
use crate::error::internal_error::InternalError;
use actix_web::http::StatusCode;
use std::time::SystemTimeError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum StorageError {
    #[error("{0}")]
    LoadFailure(String),
    #[error("IO error {0}")]
    IOError(std::io::Error),
    #[error("JSON error {0}")]
    JSONError(serde_json::Error),
    #[error("Storage Already Exists!")]
    StorageAlreadyExist,
    #[error("Repository Already Exists")]
    RepositoryAlreadyExists,
    #[error("Missing Repository")]
    RepositoryMissing,
    #[error("Unable to find Parent Directory")]
    ParentIssue,
    #[error("Internal Error: {0}")]
    InternalError(String),
}
impl From<StorageError> for APIResponse {
    fn from(storage_error: StorageError) -> Self {
        match storage_error {
            StorageError::RepositoryAlreadyExists => {
                APIResponse::from(("already exists", StatusCode::CONFLICT))
            }
            StorageError::RepositoryMissing => return APIResponse::not_found(),
            value => InternalError::from(value).into(),
        }
    }
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

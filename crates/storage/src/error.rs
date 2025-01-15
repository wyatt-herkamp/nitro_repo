use std::time::SystemTimeError;

use thiserror::Error;

use crate::{
    local::{error::LocalStorageError, LocalStorage},
    s3::S3StorageError,
    InvalidConfigType, PathCollisionError,
};

#[derive(Error, Debug)]
pub enum StorageError {
    #[error("JSON error {0}")]
    JSONError(#[from] serde_json::Error),
    #[error(transparent)]
    LocalStorageError(LocalStorageError),
    #[error(transparent)]
    S3StorageError(S3StorageError),
    #[error(transparent)]
    InvalidConfigType(#[from] InvalidConfigType),
    #[error(transparent)]
    PathCollision(#[from] PathCollisionError),
}

impl From<S3StorageError> for StorageError {
    fn from(err: S3StorageError) -> StorageError {
        match err {
            S3StorageError::PathCollision(err) => StorageError::from(err),
            _ => StorageError::S3StorageError(err),
        }
    }
}
impl From<LocalStorageError> for StorageError {
    fn from(err: LocalStorageError) -> StorageError {
        match err {
            LocalStorageError::PathCollision(err) => StorageError::from(err),
            _ => StorageError::LocalStorageError(err),
        }
    }
}

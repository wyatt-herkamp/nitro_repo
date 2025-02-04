use thiserror::Error;

use crate::{
    InvalidConfigType, PathCollisionError, local::error::LocalStorageError, s3::S3StorageError,
};
#[derive(Error, Debug, Clone, Copy, PartialEq, Eq)]
pub enum WrongFileType {
    #[error("Expected a file, but got a directory")]
    ExpectedFile,
    #[error("Expected a directory, but got a file")]
    ExpectedDirectory,
}

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
    #[error(transparent)]
    WrongFileType(#[from] WrongFileType),
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
            LocalStorageError::WrongFileType(err) => StorageError::from(err),
            _ => StorageError::LocalStorageError(err),
        }
    }
}

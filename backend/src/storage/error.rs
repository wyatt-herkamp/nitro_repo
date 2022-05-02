use crate::storage::local_storage::LocalStorageError;
use thiserror::Error;
#[derive(Error, Debug)]
pub enum StorageError {
    #[error("{0}")]
    LocalStorageError(LocalStorageError),
    #[error("{0}")]
    LoadFailure(String),
    #[error("IO error {0}")]
    IOError(std::io::Error),
    #[error("JSON error {0}")]
    JSONError(serde_json::Error),
    #[error("Storage Already Exists!")]
    StorageAlreadyExist,
}

impl From<std::io::Error> for StorageError {
    fn from(err: std::io::Error) -> StorageError {
        StorageError::IOError(err)
    }
}

impl From<LocalStorageError> for StorageError {
    fn from(err: LocalStorageError) -> StorageError {
        StorageError::LocalStorageError(err)
    }
}

impl From<serde_json::Error> for StorageError {
    fn from(err: serde_json::Error) -> StorageError {
        StorageError::JSONError(err)
    }
}
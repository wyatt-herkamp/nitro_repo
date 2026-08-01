use std::path::PathBuf;

use nr_core::storage::StoragePath;
use thiserror::Error;

use crate::{InvalidConfigType, PathCollisionError, StorageError, error::WrongFileType};

#[derive(Debug, Error)]
pub enum FsV2StorageError {
    #[error("IO Error: {0}")]
    IO(#[from] std::io::Error),
    #[error("Object encoding error: {0}")]
    Object(#[from] tux_io_encoding::fs::ObjectFileError),
    #[error("Blocking storage task failed: {0}")]
    BlockingTask(#[from] tokio::task::JoinError),
    #[error("Could not encode repository metadata: {0}")]
    Meta(#[from] postcard::Error),
    #[error(transparent)]
    InvalidConfigType(#[from] InvalidConfigType),
    #[error(transparent)]
    PathCollision(#[from] PathCollisionError),
    #[error(transparent)]
    WrongFileType(#[from] WrongFileType),
    #[error("`{0}` has no parent directory")]
    NoParentDirectory(StoragePath),
    #[error("`{0}` does not exist")]
    NotFound(StoragePath),
    #[error("The storage root `{0}` is a file")]
    RootIsAFile(PathBuf),
    #[error("The path of a FileSystemV2 storage cannot be changed once it holds artifacts")]
    PathCannotBeChanged,
}

/// Flattens the two errors that the crate models as first-class variants.
///
/// Mirrors the local and S3 backends: a path collision or a wrong file type means the same thing
/// whichever backend produced it, so callers can match on one variant instead of unwrapping a
/// backend-specific error first.
impl From<FsV2StorageError> for StorageError {
    fn from(value: FsV2StorageError) -> Self {
        match value {
            FsV2StorageError::PathCollision(err) => StorageError::PathCollision(err),
            FsV2StorageError::WrongFileType(err) => StorageError::WrongFileType(err),
            other => StorageError::FsV2StorageError(other),
        }
    }
}

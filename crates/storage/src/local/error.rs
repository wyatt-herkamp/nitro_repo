use super::{ExtensionError, ParentDirectoryDoesNotExist, PathCollisionError};

#[derive(Debug, thiserror::Error)]
pub enum LocalStorageError {
    #[error("IO Internal {0}")]
    IOError(#[from] std::io::Error),
    #[error(transparent)]
    ExtensionError(#[from] ExtensionError),
    #[error(transparent)]
    ParentDirectoryDoesNotExist(#[from] ParentDirectoryDoesNotExist),
    #[error(transparent)]
    PathCollision(#[from] PathCollisionError),
    #[error("Metadata Error {0}")]
    Postcard(#[from] postcard::Error),

    #[error("Expected A File but got a directory")]
    ExpectedFile,
    #[error("Expected A Directory but got a file")]
    ExpectedDirectory,
    #[error("Path cannot be changed")]
    PathCannotBeChanged,
    #[error("Expected a config of type Local")]
    InvalidConfigType(#[from] crate::InvalidConfigType),
}

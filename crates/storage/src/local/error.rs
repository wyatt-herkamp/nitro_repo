use crate::error::WrongFileType;

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
    #[error(transparent)]
    WrongFileType(#[from] WrongFileType),
    #[error("Path cannot be changed")]
    PathCannotBeChanged,
    #[error("Expected a config of type Local")]
    InvalidConfigType(#[from] crate::InvalidConfigType),
    #[error("Internal Unknown Error {0}")]
    Other(Box<dyn std::error::Error + Send + Sync>),
}
impl LocalStorageError {
    pub fn other(e: impl std::error::Error + Send + Sync + 'static) -> Self {
        LocalStorageError::Other(Box::new(e))
    }

    pub fn expected_file() -> Self {
        LocalStorageError::WrongFileType(WrongFileType::ExpectedFile)
    }
    pub fn expected_directory() -> Self {
        LocalStorageError::WrongFileType(WrongFileType::ExpectedDirectory)
    }
}

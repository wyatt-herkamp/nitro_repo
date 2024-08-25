mod storage_path;
use std::fmt::Display;

use derive_more::derive::{AsRef, Deref, Into};
use sqlx::Type;
pub use storage_path::*;
use thiserror::Error;
use tracing::instrument;

use crate::utils::validations;
#[derive(Debug, Error)]
pub enum InvalidStorageName {
    #[error("Storage Name is too short, must be at least 3 got {0} characters")]
    TooShort(usize),
    #[error("Storage Name is too long, must be less than 32 got {0} characters")]
    TooLong(usize),
    #[error("Storage Name contains invalid character `{0}`. Storage Names can only contain letters, numbers, `_`, and `-`")]
    InvalidCharacter(char),
}
#[derive(Debug, Type, Deref, AsRef, Clone, PartialEq, Eq, Default, Into)]
#[sqlx(transparent)]
pub struct StorageName(String);
impl StorageName {
    #[instrument(name = "StorageName::new")]
    pub fn new(storage_name: String) -> Result<Self, InvalidStorageName> {
        if storage_name.len() < 3 {
            return Err(InvalidStorageName::TooShort(storage_name.len()));
        }
        if storage_name.len() > 32 {
            return Err(InvalidStorageName::TooLong(storage_name.len()));
        }
        if let Some(bad_char) = storage_name
            .chars()
            .find(|c| !validations::valid_name_char(*c))
        {
            return Err(InvalidStorageName::InvalidCharacter(bad_char));
        }
        Ok(Self(storage_name))
    }
}
impl Display for StorageName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

validations::from_impls!(StorageName, InvalidStorageName);
/// Check if a file is a directory or a file
///
/// This trait is implemented for `Option<T>` where `T` implements `FileTypeCheck`. Will return false if `None`
pub trait FileTypeCheck: Sized {
    fn is_directory(&self) -> bool;
    fn is_file(&self) -> bool;
}

/// Implement `FileTypeCheck` for `Option<T>` where `T` implements `FileTypeCheck`
/// Will return false if `None`
impl<T> FileTypeCheck for Option<T>
where
    T: FileTypeCheck,
{
    fn is_directory(&self) -> bool {
        match self {
            Some(t) => t.is_directory(),
            None => false,
        }
    }

    fn is_file(&self) -> bool {
        match self {
            Some(t) => t.is_file(),
            None => false,
        }
    }
}

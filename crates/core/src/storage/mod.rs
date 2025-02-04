mod mime;
mod storage_path;
pub use mime::*;
use nr_macros::{NuType, SerdeViaStr};
use serde::{Deserialize, Serialize};
use sqlx::Type;
pub use storage_path::*;
use thiserror::Error;
use tracing::instrument;
use utoipa::ToSchema;

use crate::utils::validations::{self, schema_for_new_type_str, test_validations};
#[derive(Debug, Error)]
pub enum InvalidStorageName {
    #[error("Storage Name is too short, must be at least 3 got {0} characters")]
    TooShort(usize),
    #[error("Storage Name is too long, must be less than 32 got {0} characters")]
    TooLong(usize),
    #[error(
        "Storage Name contains invalid character `{0}`. Storage Names can only contain letters, numbers, `_`, and `-`"
    )]
    InvalidCharacter(char),
}
#[derive(Debug, Type, Clone, Default, SerdeViaStr, NuType)]
#[sqlx(transparent)]
pub struct StorageName(String);
schema_for_new_type_str!(StorageName, pattern = r#"^([a-zA-Z0-9_\-]{3,32}$)"#);
validations::convert_traits_to_new!(StorageName, InvalidStorageName);

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
test_validations! {
    mod storage_name_tests for StorageName {
        valid: [
            "test",
            "test-123",
            "test_123",
            "test-123_",
            "test_123-",
            "test_123-abc",
            "test_123-abc_"
        ],
        invalid: [
            "t e",
            "t"
        ]
    }
}
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
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Default, ToSchema)]
pub struct FileHashes {
    pub md5: Option<String>,
    pub sha1: Option<String>,
    pub sha2_256: Option<String>,
    pub sha3_256: Option<String>,
}

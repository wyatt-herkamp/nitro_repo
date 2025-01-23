mod content;
mod err;
mod file;
mod file_meta;

pub(crate) mod path;
pub(crate) mod utils;
pub use content::*;
use derive_more::{derive::Deref, From, Into};
pub use file::*;
pub use file_meta::*;
pub use path::{ExtensionError, ParentDirectoryDoesNotExist};
mod file_reader;
pub use file_reader::*;
use nr_core::storage::StoragePath;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use thiserror::Error;
use utoipa::ToSchema;

/// If you have two paths
/// /a/b and /a/b/c
/// Because `b` is a file and then c is treating b as a directory it would be a conflict.
#[derive(Debug, Clone, Error)]
#[error("Path {path} conflicts with {conflicts_with}")]
pub struct PathCollisionError {
    pub path: StoragePath,
    pub conflicts_with: StoragePath,
}

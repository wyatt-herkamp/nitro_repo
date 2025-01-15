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
#[derive(Debug, Clone, From, Into, Deref, ToSchema)]
#[schema(value_type = String)]
pub struct SerdeMime(pub mime::Mime);

impl Serialize for SerdeMime {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.0.to_string().serialize(serializer)
    }
}
impl<'de> Deserialize<'de> for SerdeMime {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let string = String::deserialize(deserializer)?;
        Ok(SerdeMime(string.parse().map_err(serde::de::Error::custom)?))
    }
}
/// If you have two paths
/// /a/b and /a/b/c
/// Because `b` is a file and then c is treating b as a directory it would be a conflict.
#[derive(Debug, Clone, Error)]
#[error("Path {path} conflicts with {conflicts_with}")]
pub struct PathCollisionError {
    pub path: StoragePath,
    pub conflicts_with: StoragePath,
}

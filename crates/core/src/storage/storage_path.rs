use std::{fmt::Display, path::PathBuf};

use http::Uri;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use thiserror::Error;
use tracing::instrument;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
struct StoragePathComponent(String);
/// A Storage path is a UTF-8 only path. Where the root is the base of the storage.
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct StoragePath(Vec<StoragePathComponent>);
impl StoragePath {
    pub fn parent(self) -> Self {
        let mut path = self.0;
        path.pop();
        StoragePath(path)
    }
}
impl Default for StoragePath {
    fn default() -> Self {
        StoragePath(vec![])
    }
}
impl StoragePath {
    pub fn has_extension(&self, extension: &str) -> bool {
        self.0
            .last()
            .map(|v| v.0.ends_with(extension))
            .unwrap_or(false)
    }
}
impl From<StoragePath> for PathBuf {
    fn from(value: StoragePath) -> Self {
        let mut path = PathBuf::new();
        for component in value.0 {
            path.push(component.0);
        }
        path
    }
}
impl From<&StoragePath> for PathBuf {
    fn from(value: &StoragePath) -> Self {
        let mut path = PathBuf::new();
        for component in &value.0 {
            path.push(&component.0);
        }
        path
    }
}
impl Display for StoragePath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let path = self
            .0
            .iter()
            .map(|v| v.0.as_str())
            .collect::<Vec<&str>>()
            .join("/");
        write!(f, "{}", path)
    }
}
impl From<&str> for StoragePath {
    fn from(value: &str) -> Self {
        let value = value.split("/").collect::<Vec<&str>>();
        let components = value
            .iter()
            .map(|v| StoragePathComponent(v.to_string()))
            .collect::<Vec<StoragePathComponent>>();
        StoragePath(components)
    }
}
impl From<String> for StoragePath {
    fn from(value: String) -> Self {
        StoragePath::from(value.as_str())
    }
}
impl Serialize for StoragePath {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    where
        S: Serializer,
    {
        let to_string = self.to_string();
        serializer.serialize_str(&to_string)
    }
}

impl<'de> Deserialize<'de> for StoragePath {
    fn deserialize<D>(deserializer: D) -> Result<Self, <D as Deserializer<'de>>::Error>
    where
        D: Deserializer<'de>,
    {
        let string = String::deserialize(deserializer)?;
        Ok(StoragePath::from(string))
    }
}
#[derive(Debug, Error)]
pub enum InvalidStoragePath {
    #[error("Invalid path")]
    InvalidPath,
}
impl TryFrom<Uri> for StoragePath {
    type Error = InvalidStoragePath;
    #[instrument]
    fn try_from(uri: Uri) -> Result<Self, Self::Error> {
        let path = uri.path();
        Ok(StoragePath::from(path))
    }
}

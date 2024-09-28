use std::{fmt::Display, path::PathBuf};

use http::Uri;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use thiserror::Error;
use tracing::instrument;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct StoragePathComponent(String);
impl PartialEq<&str> for StoragePathComponent {
    fn eq(&self, other: &&str) -> bool {
        self.0 == *other
    }
}
impl PartialEq<str> for StoragePathComponent {
    fn eq(&self, other: &str) -> bool {
        self.0 == other
    }
}
impl TryFrom<&str> for StoragePathComponent {
    type Error = InvalidStoragePath;
    #[instrument]
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if value.is_empty() {
            return Err(InvalidStoragePath::InvalidPath);
        }
        if value.contains('/') {
            return Err(InvalidStoragePath::InvalidPath);
        }
        Ok(StoragePathComponent(value.to_string()))
    }
}
impl From<StoragePathComponent> for String {
    fn from(value: StoragePathComponent) -> Self {
        value.0
    }
}
impl Display for StoragePathComponent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}
impl AsRef<str> for StoragePathComponent {
    fn as_ref(&self) -> &str {
        &self.0
    }
}
/// A Storage path is a UTF-8 only path. Where the root is the base of the storage.
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
#[derive(Default)]
pub struct StoragePath(Vec<StoragePathComponent>);

impl StoragePath {
    pub fn parent(self) -> Self {
        let mut path = self.0;
        path.pop();
        StoragePath(path)
    }
    pub fn number_of_components(&self) -> usize {
        self.0.len()
    }
    pub fn has_extension(&self, extension: &str) -> bool {
        self.0
            .last()
            .map(|v| v.0.ends_with(extension))
            .unwrap_or(false)
    }
    pub fn push(mut self, component: &str) -> Self {
        let components = StoragePath::from(component);
        self.0.extend(components.0);
        self
    }
    pub fn push_mut(&mut self, component: &str) {
        let components = StoragePath::from(component);
        self.0.extend(components.0);
    }
}
impl From<Vec<StoragePathComponent>> for StoragePath {
    fn from(value: Vec<StoragePathComponent>) -> Self {
        StoragePath(value)
    }
}
impl From<StoragePath> for Vec<StoragePathComponent> {
    fn from(value: StoragePath) -> Self {
        value.0
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
            .filter_map(|v| {
                if v.is_empty() {
                    None
                } else {
                    Some(StoragePathComponent(v.to_string()))
                }
            })
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

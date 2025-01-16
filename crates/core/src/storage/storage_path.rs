use std::{fmt::Display, path::PathBuf};

use http::Uri;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use thiserror::Error;
use tracing::instrument;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct StoragePathComponent(String);
impl StoragePathComponent {
    fn should_add_slash(component: &str) -> Option<bool> {
        if component.ends_with('/') {
            Some(true)
        } else if component.contains(".") {
            Some(false)
        } else {
            None
        }
    }
}
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

/// A Storage path is a UTF-8 only path. Where the root is the base of the repository.
#[derive(Debug, Clone, Hash, PartialEq, Eq, Default)]
pub struct StoragePath {
    components: Vec<StoragePathComponent>,
    trailing_slash: Option<bool>,
}
impl utoipa::__dev::ComposeSchema for StoragePath {
    fn compose(
        _: Vec<utoipa::openapi::RefOr<utoipa::openapi::schema::Schema>>,
    ) -> utoipa::openapi::RefOr<utoipa::openapi::schema::Schema> {
        utoipa::openapi::ObjectBuilder::new()
            .schema_type(utoipa::openapi::schema::SchemaType::new(
                utoipa::openapi::schema::Type::String,
            ))
            .into()
    }
}
impl utoipa::ToSchema for StoragePath {
    fn name() -> std::borrow::Cow<'static, str> {
        std::borrow::Cow::Borrowed("StoragePath")
    }
    fn schemas(
        schemas: &mut Vec<(
            String,
            utoipa::openapi::RefOr<utoipa::openapi::schema::Schema>,
        )>,
    ) {
        schemas.extend([]);
    }
}

impl StoragePath {
    /// The parent of the path is always a directory.
    pub fn parent(self) -> Self {
        let mut path = self.components;
        path.pop();
        // Every parent will contain a trailing slash.
        StoragePath {
            components: path,
            trailing_slash: Some(true),
        }
    }
    pub fn number_of_components(&self) -> usize {
        self.components.len()
    }
    pub fn has_extension(&self, extension: &str) -> bool {
        // Trailing Slashes implies that it is a directory.
        if self.trailing_slash == Some(true) {
            return false;
        }
        self.components
            .last()
            .map(|v| v.0.ends_with(extension))
            .unwrap_or(false)
    }
    pub fn push(mut self, component: &str) -> Self {
        let new_path = StoragePath::from(component);
        self.components.extend(new_path.components);
        self.trailing_slash = new_path.trailing_slash;
        self
    }
    pub fn push_mut(&mut self, component: &str) {
        let new_path = StoragePath::from(component);

        self.components.extend(new_path.components);
        self.trailing_slash = new_path.trailing_slash;
    }
    pub fn is_directory(&self) -> bool {
        self.trailing_slash == Some(true)
    }
}
impl From<Vec<StoragePathComponent>> for StoragePath {
    fn from(value: Vec<StoragePathComponent>) -> Self {
        StoragePath {
            components: value,
            trailing_slash: None,
        }
    }
}
impl From<StoragePath> for Vec<StoragePathComponent> {
    fn from(value: StoragePath) -> Self {
        value.components
    }
}
impl From<StoragePath> for PathBuf {
    fn from(value: StoragePath) -> Self {
        let mut path = PathBuf::new();
        for component in value.components {
            path.push(component.0);
        }
        path
    }
}
impl From<&StoragePath> for PathBuf {
    fn from(value: &StoragePath) -> Self {
        let mut path = PathBuf::new();
        for component in &value.components {
            path.push(&component.0);
        }
        path
    }
}
impl IntoIterator for StoragePath {
    type Item = StoragePathComponent;
    type IntoIter = <Vec<StoragePathComponent> as IntoIterator>::IntoIter;
    fn into_iter(self) -> Self::IntoIter {
        self.components.into_iter()
    }
}
impl Display for StoragePath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut path = self
            .components
            .iter()
            .map(|v| v.0.as_str())
            .collect::<Vec<&str>>()
            .join("/");
        if self.trailing_slash == Some(true) {
            path.push('/');
        }
        write!(f, "{}", path)
    }
}
impl From<&str> for StoragePath {
    fn from(value: &str) -> Self {
        let trailing_slash = StoragePathComponent::should_add_slash(value);
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
        StoragePath {
            components,
            trailing_slash,
        }
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

#[cfg(test)]
mod tests {
    use crate::storage::StoragePath;
    #[test]
    fn test_from_and_into() {
        let path = StoragePath::from("/test");
        assert_eq!(path.to_string(), "test");
        let path = StoragePath::from("test/test2");
        assert_eq!(path.to_string(), "test/test2");
        let path = StoragePath::from("test/test2/");
        assert_eq!(path.to_string(), "test/test2");
        let path = StoragePath::from("test/test2/test3");
        assert_eq!(path.to_string(), "test/test2/test3");
        let path = StoragePath::from("test/test2/test3/");
        assert_eq!(path.to_string(), "test/test2/test3");
    }
}

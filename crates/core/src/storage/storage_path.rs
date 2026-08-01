use std::{
    fmt::{Debug, Display},
    path::PathBuf,
    str::FromStr,
};

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
    /// Whether a single path segment is safe to use as a storage path component.
    ///
    /// A [StoragePath] is joined onto a storage root (a directory for the local backends, a key
    /// prefix for S3), so a component that means "go up a level" would let a request escape the
    /// repository it addresses. `.` and `..` are therefore rejected outright rather than resolved.
    ///
    /// Separators and NUL are rejected because the component is later pushed onto a `PathBuf`
    /// verbatim, where they would silently split into more components than the caller intended.
    /// The remaining control characters have no legitimate use in an artifact path.
    pub fn is_valid_component(component: &str) -> bool {
        !(component.is_empty()
            || component == "."
            || component == ".."
            || component.contains('/')
            || component.contains('\\')
            || component.contains('\0')
            || component.chars().any(char::is_control))
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
        if !Self::is_valid_component(value) {
            return Err(InvalidStoragePath::InvalidComponent(value.to_owned()));
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
#[derive(Clone, Hash, PartialEq, Eq, Default)]
pub struct StoragePath {
    components: Vec<StoragePathComponent>,
    trailing_slash: Option<bool>,
}
impl Debug for StoragePath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.to_string())
    }
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
    /// Parses a `/`-separated path, rejecting any component that is not a safe path segment.
    ///
    /// This is the constructor to use for anything a client controls — request paths, websocket
    /// messages, request bodies. Empty segments (from a leading, trailing or doubled `/`) are
    /// normalised away as usual; `.`, `..`, and segments containing a separator, NUL or a control
    /// character are errors rather than being silently dropped.
    #[instrument]
    pub fn parse(value: &str) -> Result<Self, InvalidStoragePath> {
        let trailing_slash = StoragePathComponent::should_add_slash(value);
        let components = value
            .split('/')
            // A leading/trailing/doubled slash is normal in a URL and carries no meaning here.
            .filter(|segment| !segment.is_empty())
            .map(StoragePathComponent::try_from)
            .collect::<Result<Vec<_>, _>>()?;
        Ok(StoragePath {
            components,
            trailing_slash,
        })
    }
}
impl FromStr for StoragePath {
    type Err = InvalidStoragePath;
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::parse(value)
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
/// Builds a path from a `/`-separated string, **dropping** any component that
/// [StoragePathComponent::is_valid_component] rejects.
///
/// This is the infallible constructor, used where the input is already trusted (paths read back
/// out of the database, paths assembled from validated components). It drops rather than resolves
/// unsafe components so that no caller can produce a traversing path even by mistake — but because
/// dropping silently changes what the path addresses, anything reading untrusted input should use
/// [StoragePath::parse], which reports the bad component instead.
impl From<&str> for StoragePath {
    fn from(value: &str) -> Self {
        let trailing_slash = StoragePathComponent::should_add_slash(value);
        let components = value
            .split('/')
            .filter(|v| StoragePathComponent::is_valid_component(v))
            .map(|v| StoragePathComponent(v.to_string()))
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
        // Deserialization is the boundary every client-supplied path crosses (the `{*path}` route
        // capture and the browse websocket both land here), so it validates rather than dropping.
        StoragePath::parse(&string).map_err(serde::de::Error::custom)
    }
}
#[derive(Debug, Error)]
pub enum InvalidStoragePath {
    #[error("Invalid path")]
    InvalidPath,
    #[error(
        "Invalid path component `{0}`: components may not be `.`, `..`, or contain a path separator, NUL, or control characters"
    )]
    InvalidComponent(String),
}
impl TryFrom<Uri> for StoragePath {
    type Error = InvalidStoragePath;
    #[instrument]
    fn try_from(uri: Uri) -> Result<Self, Self::Error> {
        StoragePath::parse(uri.path())
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use serde::{Deserialize, Serialize};

    use crate::storage::StoragePath;
    #[test]
    fn prefix_slash() {
        let path = StoragePath::from("/test");
        assert_eq!(path.to_string(), "test");
        let path = StoragePath::from("/test/test2");
        assert_eq!(path.to_string(), "test/test2");
        let path = StoragePath::from("/test/test2/");
        assert_eq!(path.to_string(), "test/test2/");
        let path = StoragePath::from("/test/test2/test3");
        assert_eq!(path.to_string(), "test/test2/test3");
        let path = StoragePath::from("/test/test2/test3/");
        assert_eq!(path.to_string(), "test/test2/test3/");
    }
    #[test]
    fn test_from_and_into() {
        let path = StoragePath::from("test/test2");
        assert_eq!(path.to_string(), "test/test2");
        let path = StoragePath::from("test/test2/");
        assert_eq!(path.to_string(), "test/test2/");
        let path = StoragePath::from("test/test2/test3");
        assert_eq!(path.to_string(), "test/test2/test3");
        let path = StoragePath::from("test/test2/test3/");
        assert_eq!(path.to_string(), "test/test2/test3/");
    }
    #[test]
    fn double_slash() {
        let path = StoragePath::from("test//test2");
        assert_eq!(path.to_string(), "test/test2");
        let path = StoragePath::from("test//test2/");
        assert_eq!(path.to_string(), "test/test2/");
        let path = StoragePath::from("test/test2//test3");
        assert_eq!(path.to_string(), "test/test2/test3");
        let path = StoragePath::from("test/test2//test3/");
        assert_eq!(path.to_string(), "test/test2/test3/");
    }
    /// A path that walks out of the repository root must not parse at all.
    ///
    /// These reach `parse` through the `{storage}/{repository}/{*path}` route capture, which axum
    /// percent-decodes for us — so the encoded forms below arrive here already decoded, and only
    /// the decoded form needs rejecting.
    #[test]
    fn parse_rejects_traversal() {
        let traversing = [
            "..",
            "../etc/passwd",
            "a/../../etc/passwd",
            "a/b/..",
            "/../../../../etc/shadow",
            "dev/kingtux/../../../secret",
        ];
        for path in traversing {
            let result = StoragePath::parse(path);
            assert!(
                result.is_err(),
                "`{path}` parsed to {:?} instead of being rejected",
                result.ok()
            );
        }
    }

    #[test]
    fn parse_rejects_other_unsafe_components() {
        for path in ["a/./b", ".", "a/b\\c", "a/b\0c", "a/b\nc"] {
            assert!(
                StoragePath::parse(path).is_err(),
                "`{}` should have been rejected",
                path.escape_debug()
            );
        }
    }

    /// Dots are only special as a whole component — real artifact names are full of them.
    #[test]
    fn parse_accepts_ordinary_artifact_paths() {
        let accepted = [
            "dev/kingtux/tms/1.0.0/tms-1.0.0.pom",
            "test.txt",
            "a/b.c.d/e",
            "@scope/pkg/-/pkg-1.0.0.tgz",
            "..leading-dots-are-fine",
            "trailing..",
        ];
        for path in accepted {
            assert!(
                StoragePath::parse(path).is_ok(),
                "`{path}` should have been accepted"
            );
        }
    }

    /// `parse` normalises the same leading/trailing/doubled slashes that `From<&str>` does.
    #[test]
    fn parse_matches_from_for_safe_paths() {
        for path in [
            "/test",
            "test/test2/",
            "test//test2",
            "/test/test2/test3/",
            "test.txt",
        ] {
            assert_eq!(
                StoragePath::parse(path).unwrap(),
                StoragePath::from(path),
                "`{path}` disagreed between parse and From"
            );
        }
    }

    /// The infallible constructor drops what `parse` rejects, so no caller can build a
    /// traversing path even by accident.
    #[test]
    fn from_str_drops_traversal_components() {
        assert_eq!(
            StoragePath::from("a/../../etc/passwd").to_string(),
            "a/etc/passwd"
        );
        assert_eq!(StoragePath::from("../..").to_string(), "");
        assert_eq!(StoragePath::from("a/./b").to_string(), "a/b");
    }

    /// Deserialization is the boundary client paths cross; it must reject, not drop.
    #[test]
    fn deserialize_rejects_traversal() {
        let err = serde_json::from_str::<Test>(r#"{"path":"a/../../etc/passwd"}"#);
        assert!(err.is_err(), "traversing path deserialized successfully");
    }

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
    struct Test {
        path: StoragePath,
    }
    #[test]
    fn serde() {
        let paths = vec![
            "test/test2",
            "test/test2/",
            "test/test2/test3",
            "test/test2/test3/",
            "/test/test2",
        ];
        for path in paths {
            let test = Test {
                path: StoragePath::from(path),
            };
            let serialized = serde_json::to_string(&test).unwrap();
            let deserialized: Test = serde_json::from_str(&serialized).unwrap();
            assert_eq!(test, deserialized);
        }
    }
}

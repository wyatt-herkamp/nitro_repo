use std::fmt::Display;

use derive_more::derive::{AsRef, Deref};
use schemars::JsonSchema;
use serde::Serialize;
use tracing::{instrument, trace};
use url::Url;

use crate::storage::StoragePath;

#[derive(Debug, Clone, PartialEq, Eq, JsonSchema, Deref, AsRef)]
pub struct ProxyURL(String);

impl Serialize for ProxyURL {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl<'de> serde::Deserialize<'de> for ProxyURL {
    fn deserialize<D>(deserializer: D) -> Result<ProxyURL, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;

        Ok(ProxyURL::try_from(s).map_err(serde::de::Error::custom)?)
    }
}

impl TryFrom<String> for ProxyURL {
    type Error = url::ParseError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let mut s = value;
        if s.ends_with("/") {
            s.pop();
        }
        let url = url::Url::parse(&s)?;
        trace!(url = %url, "Parsed URL");
        Ok(ProxyURL(s))
    }
}
impl Display for ProxyURL {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}
impl From<ProxyURL> for String {
    fn from(url: ProxyURL) -> String {
        url.0
    }
}
impl ProxyURL {
    /// Creates a URL from a proxyURL and a path
    #[instrument]
    pub fn add_storage_path(&self, path: StoragePath) -> Result<Url, url::ParseError> {
        let mut path = path.to_string();
        if path.starts_with("/") {
            path = path[1..].to_string();
        }
        let raw_url = format!("{}/{}", self.0, path);
        trace!(url = %raw_url, "Creating URL");
        let url = Url::parse(&format!("{}/{}", self.0, path))?;
        Ok(url)
    }
}

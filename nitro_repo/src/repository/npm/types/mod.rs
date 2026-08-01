pub mod request;

use ahash::HashMap;
use chrono::{DateTime, FixedOffset};
use http::HeaderName;
use request::PublishVersion;
use serde::{Deserialize, Serialize};
use serde_json::Value;
mod name;
mod publish;
pub use name::{InvalidNPMPackageName, NPMPackageName};
pub use publish::*;
pub const NPM_COMMAND_HEADER: HeaderName = HeaderName::from_static("npm-command");
#[derive(Debug, Clone)]
pub struct RegistryResponse {
    pub db_name: String,
    pub engine: String,
    pub doc_count: u64,
    pub doc_del_count: u64,
    pub update_seq: u64,
    pub purge_seq: u64,
    pub compact_running: bool,
    // TODO: Add more fields
}
/// The packument — what `GET /{package}` returns.
///
/// `npm view`, `npm info` and the installer all read fields that were simply absent before:
/// `_rev` (which `npm unpublish` and `npm dist-tag` send back as a precondition), `maintainers`,
/// `readme`, `license`, `keywords`, `repository`, `bugs` and `homepage`. The types for several of
/// these already existed in this module and were never referenced.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NpmRegistryPackageResponse {
    #[serde(rename = "_id")]
    pub id: String,
    /// CouchDB revision. npm echoes it back on unpublish and dist-tag writes, and refuses some
    /// operations without it.
    #[serde(rename = "_rev")]
    pub rev: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(rename = "dist-tags")]
    pub dist_tags: HashMap<String, String>,
    pub versions: HashMap<String, PublishVersion>,
    pub time: HashMap<String, String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub maintainers: Vec<Maintainers>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub readme: Option<String>,
    #[serde(rename = "readmeFilename", skip_serializing_if = "Option::is_none")]
    pub readme_filename: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub license: Option<Value>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub keywords: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repository: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bugs: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub homepage: Option<Value>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NPMRegistryPackageTime {
    pub created: String,
    pub modified: DateTime<FixedOffset>,
    #[serde(flatten)]
    pub versions: HashMap<String, String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Maintainers {
    pub name: String,
    pub email: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bugs {
    pub url: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageFile {
    pub name: String,
    pub version: String,
    pub main: Option<String>,
    pub module: Option<String>,
    #[serde(flatten)]
    pub other: HashMap<String, Value>,
}

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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NpmRegistryPackageResponse {
    #[serde(rename = "_id")]
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    #[serde(rename = "dist-tags")]
    pub dist_tags: HashMap<String, String>,
    pub versions: HashMap<String, PublishVersion>,
    pub time: HashMap<String, String>,
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

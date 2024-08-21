//! NPM Registry Implementation
//!
//! Documentation for NPM: https://github.com/npm/registry/blob/main/docs/REGISTRY-API.md

use ahash::HashMap;
use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};
use serde_json::Value;
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
pub struct NpmRegistryPackage {
    #[serde(rename = "_id")]
    pub id: String,
    #[serde(rename = "_rev")]
    pub rev: String,
    pub name: String,
    pub description: Option<String>,
    #[serde(rename = "dist-tags")]
    pub dist_tags: HashMap<String, String>,
    pub versions: HashMap<String, Value>,
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

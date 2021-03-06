use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;
#[derive(Debug, Serialize, Deserialize, Clone,Default)]
pub  struct NPMSettings{

}
#[derive(Debug, Serialize, Deserialize)]
pub struct LoginRequest {
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginResponse {
    pub ok: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Attachment {
    pub content_type: String,
    pub data: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PublishRequest {
    pub name: String,
    pub _attachments: HashMap<String, Value>,
    pub versions: HashMap<String, Version>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Dist {
    pub integrity: String,
    pub shasum: String,
    pub tarball: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Version {
    pub version: String,
    pub name: String,
    pub dist: Dist,
    #[serde(flatten)]
    pub other: HashMap<String, Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DistTags {
    pub latest: String,
}

impl From<String> for DistTags {
    fn from(value: String) -> Self {
        DistTags { latest: value }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NPMTimes {
    pub created: String,
    pub modified: String,
    #[serde(flatten)]
    pub times: HashMap<String, String>,
}

pub type NPMVersions = HashMap<String, Version>;

#[derive(Debug, Serialize, Deserialize)]
pub struct GetResponse {
    #[serde(flatten)]
    pub version_data: Version,
    pub versions: NPMVersions,
    pub times: NPMTimes,
    #[serde(rename = "dist-tags")]
    pub dist_tags: DistTags,
}

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

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

#[derive(Debug, Serialize, Deserialize)]
pub struct Version {
    pub version: String,
    pub name: String,
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
pub struct GetResponse {
    pub id: String,
    pub name: String,
    pub versions: HashMap<String, Version>,
    pub times: HashMap<String, String>,
    #[serde(rename = "dist-tags")]
    pub dist_tags: DistTags,
    #[serde(flatten)]
    pub other: HashMap<String, Value>,
}

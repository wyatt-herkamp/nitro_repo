use std::collections::HashMap;
use std::io::Bytes;
use std::iter::Map;

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
    pub versions: HashMap<String, Value>,
}
use std::fmt::Debug;

use ahash::HashMap;
use base64::{engine::general_purpose::STANDARD, Engine};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::repository::npm::NPMRegistryError;

use super::request::PublishVersion;

#[derive(Deserialize, Serialize, Clone, PartialEq)]
pub struct PublishAttachment {
    /// Content-Type of the attachment
    /// Almost always `application/octet-stream`
    pub content_type: String,
    /// Raw Data of the attachment
    pub data: String,
    /// Length of the attachment
    pub length: usize,
}
impl Debug for PublishAttachment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PublishAttachment")
            .field("content_type", &self.content_type)
            .field("length", &self.length)
            .finish()
    }
}
impl PublishAttachment {
    pub fn read_data(self) -> Result<Vec<u8>, NPMRegistryError> {
        let mut data = Vec::with_capacity(self.length);
        STANDARD
            .decode_vec(self.data, &mut data)
            .map_err(NPMRegistryError::InvalidPackageAttachment)?;
        Ok(data)
    }
    pub fn new(data: Vec<u8>, content_type: String) -> Self {
        let length = data.len();
        let data = STANDARD.encode(data);
        Self {
            content_type,
            data,
            length,
        }
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PublishRequest {
    pub name: String,
    pub versions: HashMap<String, PublishVersion>,
    #[serde(flatten)]
    pub other: HashMap<String, Value>,
    #[serde(rename = "_attachments")]
    pub attachments: HashMap<String, PublishAttachment>,
}

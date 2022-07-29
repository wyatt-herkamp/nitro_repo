use crate::api::Auth;
use reqwest::header::HeaderValue;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UserConfig {
    #[serde(default)]
    pub repositories: HashMap<String, RepositoryInstance>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryInstance {
    pub url: String,
    pub token: String,
    pub name: String,
    pub token_uuid: Uuid,
    pub expiration: i64,
}
impl Auth for RepositoryInstance {
    fn get_as_header(&self) -> HeaderValue {
        HeaderValue::from_str(&format!("Bearer {}", self.token)).unwrap()
    }
}

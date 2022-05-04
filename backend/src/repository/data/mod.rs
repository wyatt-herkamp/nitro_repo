use std::fmt::Debug;

use serde::{Deserialize, Serialize};

use serde_json::Value;

use crate::repository::error::RepositoryError;

use crate::repository::settings::frontend::Frontend;
use crate::repository::settings::security::{SecurityRules, Visibility};
use crate::repository::settings::webhook::Webhook;
use crate::repository::settings::{Policy, FRONTEND_CONFIG, WEBHOOK_CONFIG};
use crate::storage::models::Storage;
use async_trait::async_trait;
use serde::de::DeserializeOwned;

/// Types of Repositories that can exist.
#[derive(Serialize, Deserialize, Clone, Debug, strum_macros::Display, strum_macros::EnumString)]
pub enum RepositoryType {
    Maven,
    NPM,
}

/// The Basic Repository Config
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RepositoryConfig {
    /// Repository Name
    pub name: String,
    /// The Type of Repository
    pub repository_type: RepositoryType,
    /// Storage.
    pub storage: String,
    /// Visibility.
    #[serde(default)]
    pub visibility: Visibility,
    /// Rather or not the Repository is active. Meaning it can be pulled or pushed
    #[serde(default = "default")]
    pub active: bool,
    ///The versioning policy for the Repository
    #[serde(default)]
    pub policy: Policy,
    /// When it was created
    pub created: i64,
}
impl RepositoryConfig {
    /// Pull the Frontend Config
    pub async fn get_frontend_config(
        &self,
        storage: &Box<dyn Storage>,
    ) -> Result<Option<Frontend>, RepositoryError> {
        let option = storage.get_file(self, FRONTEND_CONFIG).await?;
        if option.is_none() {
            return Ok(None);
        }
        serde_json::from_slice(option.unwrap().as_slice())
            .map(Some)
            .map_err(RepositoryError::from)
    }
    /// Update the frontend config
    async fn save_frontend_config(
        &self,
        storage: &Box<dyn Storage>,
        frontend: Option<Frontend>,
    ) -> Result<(), RepositoryError> {
        if frontend.is_none() {
            // Treats a disable
            storage.delete_file(self, FRONTEND_CONFIG).await?;
        }
        let value = serde_json::to_string(&frontend.unwrap())?;
        storage
            .save_file(self, value.as_bytes(), FRONTEND_CONFIG)
            .await
            .map_err(RepositoryError::from)
    }
}

fn default() -> bool {
    true
}

pub trait RepositorySetting: Send + Sync + Clone + Debug + Serialize + DeserializeOwned {}

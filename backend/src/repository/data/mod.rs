use std::fmt::Debug;

use serde::{Deserialize, Serialize};

use serde_json::Value;

use crate::repository::error::RepositoryError;
use crate::repository::maven::models::MavenSettings;
use crate::repository::npm::models::NPMSettings;
use crate::repository::settings::frontend::Frontend;
use crate::repository::settings::security::SecurityRules;
use crate::repository::settings::webhook::Webhook;
use crate::repository::settings::{Policy, FRONTEND_CONFIG, WEBHOOK_CONFIG};
use crate::storage::models::Storage;
use async_trait::async_trait;

pub trait RepositorySetting: Send + Sync + Clone + Debug + TryFrom<Value> + Serialize {}

impl RepositorySetting for serde_json::Value {}

#[async_trait]
pub trait RepositoryDataType: Send + Sync + Sized {
    fn get_name(&self) -> &str;
    fn get_storage_name(&self) -> &str;
    fn get_repository_value(&self) -> &RepositoryValue;

    async fn get_webhook_config(
        &self,
        storage: &Storage,
    ) -> Result<Option<Webhook>, RepositoryError> {
        let option = storage.get_file(self, WEBHOOK_CONFIG).await?;
        if option.is_none() {
            return Ok(None);
        }
        serde_json::from_slice(option.unwrap().as_slice())
            .map(Some)
            .map_err(RepositoryError::from)
    }
    async fn save_webhook_config(
        &self,
        storage: &Storage,
        webhook: Option<Webhook>,
    ) -> Result<(), RepositoryError> {
        if webhook.is_none() {
            // Treats a disable
            storage.delete_file(self, WEBHOOK_CONFIG).await?;
        }
        let value = serde_json::to_string(&webhook.unwrap())?;
        storage
            .save_file(self, value.as_bytes(), WEBHOOK_CONFIG)
            .await
            .map_err(RepositoryError::from)
    }
    async fn get_frontend_config(
        &self,
        storage: &Storage,
    ) -> Result<Option<Frontend>, RepositoryError> {
        let option = storage.get_file(self, FRONTEND_CONFIG).await?;
        if option.is_none() {
            return Ok(None);
        }
        serde_json::from_slice(option.unwrap().as_slice())
            .map(Some)
            .map_err(RepositoryError::from)
    }
    async fn save_frontend_config(
        &self,
        storage: &Storage,
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
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RepositoryValue {
    pub name: String,
    pub repository_type: RepositoryType,
    pub storage: String,
    pub created: i64,
}

#[async_trait]
impl RepositoryDataType for RepositoryValue {
    fn get_name(&self) -> &str {
        self.name.as_str()
    }

    fn get_storage_name(&self) -> &str {
        self.storage.as_str()
    }

    fn get_repository_value(&self) -> &RepositoryValue {
        self
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RepositoryConfig<T: RepositorySetting> {
    pub init_values: RepositoryValue,
    pub main_config: RepositoryMainConfig<T>,
}

#[async_trait]
impl<RS: RepositorySetting> RepositoryDataType for RepositoryConfig<RS> {
    fn get_name(&self) -> &str {
        self.init_values.name.as_str()
    }

    fn get_storage_name(&self) -> &str {
        self.init_values.storage.as_str()
    }

    fn get_repository_value(&self) -> &RepositoryValue {
        &self.init_values
    }
}

#[async_trait]
impl<'a, RS: RepositorySetting> RepositoryDataType for &'a RepositoryConfig<RS> {
    fn get_name(&self) -> &str {
        self.init_values.name.as_str()
    }
    fn get_storage_name(&self) -> &str {
        self.init_values.storage.as_str()
    }
    fn get_repository_value(&self) -> &RepositoryValue {
        &self.init_values
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RepositoryMainConfig<T: RepositorySetting> {
    pub repository_type_settings: T,
    pub security: SecurityRules,
    #[serde(default = "default")]
    pub active: bool,
    #[serde(default)]
    pub policy: Policy,
}

impl<T: RepositorySetting + Serialize> RepositoryMainConfig<T> {
    pub async fn update(&self, storage: &Storage) -> Result<(), RepositoryError> {
        storage
            .update_repository(self.clone())
            .await
            .map_err(RepositoryError::from)
    }
}
fn default() -> bool {
    true
}

#[derive(Serialize, Deserialize, Clone, Debug, strum_macros::Display, strum_macros::EnumString)]
pub enum RepositoryType {
    Maven,
    NPM,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum RepositoryTypeSetting {
    Maven(MavenSettings),
    NPM(NPMSettings),
}

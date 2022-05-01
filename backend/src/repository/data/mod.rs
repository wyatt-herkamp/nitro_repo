use std::fmt::Debug;
use actix_web::HttpRequest;
use actix_web::web::Bytes;
use sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};
use serde::de::DeserializeOwned;
use serde_json::Value;

use crate::repository::maven::models::MavenSettings;
use crate::repository::error::RepositoryError;
use crate::repository::npm::models::NPMSettings;
use crate::repository::settings::frontend::Frontend;
use crate::repository::settings::security::SecurityRules;
use crate::repository::settings::webhook::Webhook;
use crate::storage::models::Storage;
use async_trait::async_trait;
use crate::repository::settings::Policy;

pub trait RepositorySetting: Send + Sync + Clone + Debug + TryFrom<Value> + Serialize {}

impl RepositorySetting for serde_json::Value {}

#[async_trait]
pub trait RepositoryDataType: Send + Sync + Sized {
    fn get_name(&self) -> &str;
    fn get_storage_name(&self) -> &str;
    fn get_repository_value(&self) -> &RepositoryValue;

    async fn get_webhook_config(&self, storage: &Storage) -> Result<Option<Webhook>, RepositoryError> {
        let option = storage.get_file(self, ".nitro_repo/webhook.json").await?;
        if option.is_none() {
            return Ok(None);
        }
        serde_json::from_slice(option.unwrap().as_slice()).map(|value| Some(value)).map_err(RepositoryError::from)
    }
    async fn get_frontend_config(&self, storage: &Storage) -> Result<Option<Frontend>, RepositoryError> {
        let option = storage.get_file(self, ".nitro_repo/frontend.json").await?;
        if option.is_none() {
            return Ok(None);
        }
        serde_json::from_slice(option.unwrap().as_slice()).map(|value| Some(value)).map_err(RepositoryError::from)
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
        return self.name.as_str();
    }

    fn get_storage_name(&self) -> &str {
        return self.storage.as_str();
    }

    fn get_repository_value(&self) -> &RepositoryValue {
        return self;
    }
}

#[derive(Clone, Debug)]
pub struct RepositoryConfig<T: RepositorySetting> {
    pub init_values: RepositoryValue,
    pub main_config: RepositoryMainConfig<T>,
}

#[async_trait]
impl<RS: RepositorySetting> RepositoryDataType for RepositoryConfig<RS> {
    fn get_name(&self) -> &str {
        return self.init_values.name.as_str();
    }

    fn get_storage_name(&self) -> &str {
        return self.init_values.storage.as_str();
    }

    fn get_repository_value(&self) -> &RepositoryValue {
        return &self.init_values;
    }
}

#[async_trait]
impl<'a, RS: RepositorySetting> RepositoryDataType for &'a RepositoryConfig<RS> {
    fn get_name(&self) -> &str {
        return self.init_values.name.as_str();
    }
    fn get_storage_name(&self) -> &str {
        return self.init_values.storage.as_str();
    }
    fn get_repository_value(&self) -> &RepositoryValue {
        return &self.init_values;
    }
}

impl<T: RepositorySetting> RepositoryConfig<T> {
    pub async fn get_webhook_config(&self, storage: &Storage) -> Result<Option<Webhook>, RepositoryError> {
        let option = storage.get_file(self, ".nitro_repo/webhook.json").await?;
        if option.is_none() {
            return Ok(None);
        }
        serde_json::from_slice(option.unwrap().as_slice()).map(|value| Some(value)).map_err(RepositoryError::from)
    }
    pub async fn get_frontend_config(&self, storage: &Storage) -> Result<Option<Frontend>, RepositoryError> {
        let option = storage.get_file(self, ".nitro_repo/frontend.json").await?;
        if option.is_none() {
            return Ok(None);
        }
        serde_json::from_slice(option.unwrap().as_slice()).map(|value| Some(value)).map_err(RepositoryError::from)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RepositoryMainConfig<T: RepositorySetting> {
    pub repository_type_settings: T,
    pub security: SecurityRules,
    #[serde(default = "default")]
    pub active: bool,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub policy: Policy,
}
fn default() -> bool {
    true
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum RepositoryType {
    Maven,
    NPM,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum RepositoryTypeSetting {
    Maven(MavenSettings),
    NPM(NPMSettings),
}


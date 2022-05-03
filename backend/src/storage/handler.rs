use std::fmt::Write;

use serde::{Deserialize, Serialize};

use crate::storage::error::StorageError;
use crate::storage::local_storage::LocalStorage;
use crate::storage::models::StorageType;
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageHandlerFactory {
    pub storage_type: String,
    pub config: Value,
}

impl StorageHandler {
    pub async fn load(factory: StorageHandlerFactory) -> Result<StorageHandler, StorageError> {
        match factory.storage_type.as_str() {
            "local" => Ok(StorageHandler::LocalStorage(
                LocalStorage::load(factory.config).await?,
            )),
            _ => Err(StorageError::LoadFailure(
                "Unable to find storage type".to_string(),
            )),
        }
    }
    pub fn save_value(&self) -> Result<StorageHandlerFactory, StorageError> {
        let (storage_type, config) = match self {
            StorageHandler::LocalStorage(local) => (
                "local".to_string(),
                serde_json::to_value(local.config.clone())?,
            ),
        };
        Ok(StorageHandlerFactory {
            storage_type,
            config,
        })
    }
}

impl Drop for StorageHandler {
    fn drop(&mut self) {
        match self {
            StorageHandler::LocalStorage(local) => {
                local.unload();
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum StorageHandler {
    LocalStorage(LocalStorage),
}

use crate::repository::REPOSITORY_CONF;
use crate::storage::models::{
    Storage, StorageConfig, StorageFactory, StorageFile, StorageFileResponse, StorageSaver,
    StorageStatus, StorageType,
};
use crate::storage::STORAGE_CONFIG;
use crate::utils::get_current_time;

use log::{debug, trace, warn};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;

use std::io::{Read, Write};
use std::path::PathBuf;

use std::sync::Arc;

use crate::repository::data::{RepositoryConfig, RepositorySetting, RepositoryType};
use crate::storage::error::StorageError;
use async_trait::async_trait;
use serde::de::DeserializeOwned;
use serde_json::Value;
use thiserror::Error;
use tokio::fs::{create_dir_all, read_to_string, remove_file, File, OpenOptions};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::{RwLock, RwLockReadGuard};

/// This is a storage that is here to represent a storage that failed to load from the config stage
#[derive(Debug)]
pub struct BadStorage {
    pub factory: StorageFactory,
    pub status: StorageStatus,
}
impl BadStorage {
    pub fn create(factory: StorageFactory, error: StorageError) -> Box<dyn Storage> {
        return Box::new(BadStorage {
            factory,
            status: StorageStatus::CreateError(error),
        });
    }
}
#[async_trait]
impl Storage for BadStorage {
    fn new(_: StorageFactory) -> Result<Box<dyn Storage>, (StorageError, StorageFactory)>
    where
        Self: Sized,
    {
        panic!("This should not be called!")
    }
    async fn load(&mut self) -> Result<(), StorageError> {
        warn!("Unable to load Storage Error {}", self.status);
        Ok(())
    }

    fn unload(&mut self) -> Result<(), StorageError> {
        warn!("Unloaded the Bad Storage");
        Ok(())
    }

    fn config_for_saving(&self) -> StorageSaver {
        return StorageSaver {
            storage_type: self.factory.storage_type.clone(),
            generic_config: self.factory.generic_config.clone(),
            handler_config: self.factory.handler_config.clone(),
        };
    }

    fn storage_config(&self) -> &StorageConfig {
        &self.factory.generic_config
    }

    fn impl_config(&self) -> Value {
        self.factory.handler_config.clone()
    }

    fn storage_type(&self) -> &StorageType {
        &self.factory.storage_type
    }

    fn status(&self) -> &StorageStatus {
        return &self.status;
    }

    async fn create_repository(
        &self,
        name: String,
        repository_type: RepositoryType,
    ) -> Result<(), StorageError> {
        panic!("This should not be called!")
    }

    async fn delete_repository(
        &self,
        repository: &RepositoryConfig,
        delete_files: bool,
    ) -> Result<(), StorageError> {
        panic!("This should not be called!")
    }

    async fn get_repositories(&self) -> Result<Vec<RepositoryConfig>, StorageError> {
        panic!("This should not be called!")
    }

    async fn get_repository(
        &self,
        repository: &str,
    ) -> Result<Option<RwLockReadGuard<'_, RepositoryConfig>>, StorageError> {
        panic!("This should not be called!")
    }

    async fn update_repository(&self, repository: RepositoryConfig) -> Result<(), StorageError> {
        panic!("This should not be called!")
    }

    async fn update_repository_config(
        &self,
        repository: &RepositoryConfig,
        file: &str,
        data: &Option<Value>,
    ) -> Result<(), StorageError> {
        panic!("This should not be called!")
    }

    async fn get_repository_config(
        &self,
        repository: &RepositoryConfig,
        file: &str,
    ) -> Result<Option<Value>, StorageError> {
        panic!("This should not be called!")
    }

    async fn save_file(
        &self,
        repository: &RepositoryConfig,
        file: &[u8],
        location: &str,
    ) -> Result<(), StorageError> {
        panic!("This should not be called!")
    }

    async fn delete_file(
        &self,
        repository: &RepositoryConfig,
        location: &str,
    ) -> Result<(), StorageError> {
        panic!("This should not be called!")
    }

    async fn get_file_as_response(
        &self,
        repository: &RepositoryConfig,
        location: &str,
    ) -> Result<Option<StorageFileResponse>, StorageError> {
        panic!("This should not be called!")
    }

    async fn get_file_information(
        &self,
        repository: &RepositoryConfig,
        location: &str,
    ) -> Result<Option<StorageFile>, StorageError> {
        panic!("This should not be called!")
    }

    async fn get_file(
        &self,
        repository: &RepositoryConfig,
        location: &str,
    ) -> Result<Option<Vec<u8>>, StorageError> {
        panic!("This should not be called!")
    }
}

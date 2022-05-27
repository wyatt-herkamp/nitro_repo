use crate::storage::models::{
    Storage, StorageConfig, StorageFactory, StorageSaver, StorageStatus, StorageType,
};

use log::warn;

use crate::repository::data::{RepositoryConfig, RepositoryType};
use crate::storage::error::StorageError;
use async_trait::async_trait;

use serde_json::Value;

use crate::storage::file::{StorageFile, StorageFileResponse};
use tokio::sync::RwLockReadGuard;


/// This is a storage that is here to represent a storage that failed to load from the config stage
#[derive(Debug)]
pub struct BadStorage {
    pub factory: StorageFactory,
    pub status: StorageStatus,
}
impl BadStorage {
    pub fn create(factory: StorageFactory, error: StorageError) -> BadStorage {
       BadStorage {
            factory,
            status: StorageStatus::CreateError(error),
        }
    }
}
#[async_trait]
impl Storage for BadStorage {
    fn new(_: StorageFactory) -> Result<Self, (StorageError, StorageFactory)>
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
        StorageSaver {
            storage_type: self.factory.storage_type.clone(),
            generic_config: self.factory.generic_config.clone(),
            handler_config: self.factory.handler_config.clone(),
        }
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
        &self.status
    }

    async fn create_repository(
        &self,
        _name: String,
        _repository_type: RepositoryType,
    ) -> Result<(), StorageError> {
        panic!("This should not be called!")
    }

    async fn delete_repository(
        &self,
        _repository: &RepositoryConfig,
        _delete_files: bool,
    ) -> Result<(), StorageError> {
        panic!("This should not be called!")
    }

    async fn get_repositories(&self) -> Result<Vec<RepositoryConfig>, StorageError> {
        panic!("This should not be called!")
    }

    async fn get_repository(
        &self,
        _repository: &str,
    ) -> Result<Option<RwLockReadGuard<'_, RepositoryConfig>>, StorageError> {
        panic!("This should not be called!")
    }

    async fn update_repository(&self, _repository: RepositoryConfig) -> Result<(), StorageError> {
        panic!("This should not be called!")
    }

    async fn update_repository_config(
        &self,
        _repository: &RepositoryConfig,
        _file: &str,
        _data: &Option<Value>,
    ) -> Result<(), StorageError> {
        panic!("This should not be called!")
    }

    async fn get_repository_config(
        &self,
        _repository: &RepositoryConfig,
        _file: &str,
    ) -> Result<Option<Value>, StorageError> {
        panic!("This should not be called!")
    }

    async fn save_file(
        &self,
        _repository: &RepositoryConfig,
        _file: &[u8],
        _location: &str,
    ) -> Result<bool, StorageError> {
        panic!("This should not be called!")
    }

    async fn delete_file(
        &self,
        _repository: &RepositoryConfig,
        _location: &str,
    ) -> Result<(), StorageError> {
        panic!("This should not be called!")
    }

    async fn get_file_as_response(
        &self,
        _repository: &RepositoryConfig,
        _location: &str,
    ) -> Result<StorageFileResponse, StorageError> {
        panic!("This should not be called!")
    }

    async fn get_file_information(
        &self,
        _repository: &RepositoryConfig,
        _location: &str,
    ) -> Result<Option<StorageFile>, StorageError> {
        panic!("This should not be called!")
    }

    async fn get_file(
        &self,
        _repository: &RepositoryConfig,
        _location: &str,
    ) -> Result<Option<Vec<u8>>, StorageError> {
        panic!("This should not be called!")
    }
}

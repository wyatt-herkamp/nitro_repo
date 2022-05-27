use serde_json::Value;
use tokio::sync::RwLockReadGuard;
use crate::repository::data::{RepositoryConfig, RepositoryType};
use crate::storage::error::StorageError;
use crate::storage::file::{StorageFile, StorageFileResponse};
use crate::storage::local_storage::LocalStorage;
use crate::storage::models::{Storage, StorageConfig, StorageFactory, StorageSaver, StorageStatus, StorageType};
use async_trait::async_trait;
use crate::storage::bad_storage::BadStorage;

pub mod bad_storage;
pub mod error;
pub mod file;
pub mod local_storage;
pub mod models;
pub mod multi;

pub static STORAGES_CONFIG: &str = "storages.nitro_repo";
pub static STORAGE_CONFIG: &str = "storage.nitro_repo";

#[derive(Debug)]
pub enum DynamicStorage {
    LocalStorage(LocalStorage),
    BadStorage(BadStorage),
}

#[async_trait]
impl Storage for DynamicStorage {
    fn new(_config: StorageFactory) -> Result<DynamicStorage, (StorageError, StorageFactory)> where Self: Sized {
        panic!("Illegal Call")
    }

    async fn load(&mut self) -> Result<(), StorageError> {
        match self {
            DynamicStorage::LocalStorage(local) => { local.load().await }
            DynamicStorage::BadStorage(bad) => {
                bad.load().await
            }
        }
    }

    fn unload(&mut self) -> Result<(), StorageError> {
        match self {
            DynamicStorage::LocalStorage(local) => { local.unload() }
            DynamicStorage::BadStorage(bad) => {
                bad.unload()
            }
        }
    }

    fn config_for_saving(&self) -> StorageSaver {
        match self {
            DynamicStorage::LocalStorage(local) => { local.config_for_saving() }
            DynamicStorage::BadStorage(bad) => {
                bad.config_for_saving()
            }
        }
    }

    fn storage_config(&self) -> &StorageConfig {
        match self {
            DynamicStorage::LocalStorage(local) => { local.storage_config() }
            DynamicStorage::BadStorage(bad) => {
                bad.storage_config()
            }
        }
    }

    fn impl_config(&self) -> Value {
        match self {
            DynamicStorage::LocalStorage(local) => { local.impl_config() }
            DynamicStorage::BadStorage(bad) => {
                bad.impl_config()
            }
        }
    }

    fn storage_type(&self) -> &StorageType {
        match self {
            DynamicStorage::LocalStorage(local) => { local.storage_type() }
            DynamicStorage::BadStorage(bad) => {
                bad.storage_type()
            }
        }
    }

    fn status(&self) -> &StorageStatus {
        match self {
            DynamicStorage::LocalStorage(local) => { local.status() }
            DynamicStorage::BadStorage(bad) => {
                bad.status()
            }
        }
    }

    async fn create_repository(&self, name: String, repository_type: RepositoryType) -> Result<(), StorageError> {
        match self {
            DynamicStorage::LocalStorage(local) => { local.create_repository(name, repository_type).await }
            DynamicStorage::BadStorage(bad) => {
                bad.create_repository(name, repository_type).await
            }
        }
    }

    async fn delete_repository(&self, repository: &RepositoryConfig, delete_files: bool) -> Result<(), StorageError> {
        match self {
            DynamicStorage::LocalStorage(local) => { local.delete_repository(repository, delete_files).await }
            DynamicStorage::BadStorage(bad) => {
                bad.delete_repository(repository, delete_files).await
            }
        }
    }

    async fn get_repositories(&self) -> Result<Vec<RepositoryConfig>, StorageError> {
        match self {
            DynamicStorage::LocalStorage(local) => { local.get_repositories().await }
            DynamicStorage::BadStorage(bad) => {
                bad.get_repositories().await
            }
        }
    }

    async fn get_repository(&self, repository: &str) -> Result<Option<RwLockReadGuard<'_, RepositoryConfig>>, StorageError> {
        match self {
            DynamicStorage::LocalStorage(local) => { local.get_repository(repository).await }
            DynamicStorage::BadStorage(bad) => {
                bad.get_repository(repository).await
            }
        }
    }

    async fn update_repository(&self, repository: RepositoryConfig) -> Result<(), StorageError> {
        match self {
            DynamicStorage::LocalStorage(local) => { local.update_repository(repository).await }
            DynamicStorage::BadStorage(bad) => {
                bad.update_repository(repository).await
            }
        }
    }

    async fn update_repository_config(&self, repository: &RepositoryConfig, file: &str, data: &Option<Value>) -> Result<(), StorageError> {
        match self {
            DynamicStorage::LocalStorage(local) => { local.update_repository_config(repository, file, data).await }
            DynamicStorage::BadStorage(bad) => {
                bad.update_repository_config(repository, file, data).await
            }
        }
    }

    async fn get_repository_config(&self, repository: &RepositoryConfig, file: &str) -> Result<Option<Value>, StorageError> {
        match self {
            DynamicStorage::LocalStorage(local) => { local.get_repository_config(repository, file).await }
            DynamicStorage::BadStorage(bad) => {
                bad.get_repository_config(repository, file).await
            }
        }
    }

    async fn save_file(&self, repository: &RepositoryConfig, file: &[u8], location: &str) -> Result<bool, StorageError> {
        match self {
            DynamicStorage::LocalStorage(local) => { local.save_file(repository, file, location).await }
            DynamicStorage::BadStorage(bad) => {
                bad.save_file(repository, file, location).await
            }
        }
    }

    async fn delete_file(&self, repository: &RepositoryConfig, location: &str) -> Result<(), StorageError> {
        match self {
            DynamicStorage::LocalStorage(local) => { local.delete_file(repository, location).await}
            DynamicStorage::BadStorage(bad) => {
                bad.delete_file(repository, location).await
            }
        }
    }

    async fn get_file_as_response(&self, repository: &RepositoryConfig, location: &str) -> Result<StorageFileResponse, StorageError> {
        match self {
            DynamicStorage::LocalStorage(local) => { local.get_file_as_response(repository, location).await }
            DynamicStorage::BadStorage(bad) => {
                bad.get_file_as_response(repository, location).await
            }
        }
    }

    async fn get_file_information(&self, repository: &RepositoryConfig, location: &str) -> Result<Option<StorageFile>, StorageError> {
        match self {
            DynamicStorage::LocalStorage(local) => { local.get_file_information(repository, location).await }
            DynamicStorage::BadStorage(bad) => {
                bad.get_file_information(repository, location).await
            }
        }
    }

    async fn get_file(&self, repository: &RepositoryConfig, location: &str) -> Result<Option<Vec<u8>>, StorageError> {
        match self {
            DynamicStorage::LocalStorage(local) => { local.get_file(repository, location).await }
            DynamicStorage::BadStorage(bad) => {
                bad.get_file(repository, location).await
            }
        }
    }
}
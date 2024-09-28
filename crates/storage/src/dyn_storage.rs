use nr_core::storage::StoragePath;
use serde_json::Value;
use uuid::Uuid;

use crate::{
    local::{LocalStorage, LocalStorageFactory},
    FileContent, Storage, StorageError, StorageFactory, StorageTypeConfig,
};
#[derive(Debug, Clone)]
pub enum DynStorage {
    Local(LocalStorage),
}
impl Storage for DynStorage {
    async fn unload(&self) -> Result<(), StorageError> {
        match self {
            DynStorage::Local(storage) => storage.unload().await,
        }
    }

    fn storage_config(&self) -> crate::BorrowedStorageConfig<'_> {
        match self {
            DynStorage::Local(storage) => storage.storage_config(),
        }
    }

    async fn save_file(
        &self,
        repository: Uuid,
        file: FileContent,
        location: &StoragePath,
    ) -> Result<(usize, bool), StorageError> {
        match self {
            DynStorage::Local(storage) => storage.save_file(repository, file, location).await,
        }
    }

    async fn delete_file(
        &self,
        repository: Uuid,
        location: &StoragePath,
    ) -> Result<(), StorageError> {
        match self {
            DynStorage::Local(storage) => storage.delete_file(repository, location).await,
        }
    }

    async fn get_file_information(
        &self,
        repository: Uuid,
        location: &StoragePath,
    ) -> Result<Option<crate::StorageFileMeta>, StorageError> {
        match self {
            DynStorage::Local(storage) => storage.get_file_information(repository, location).await,
        }
    }

    async fn open_file(
        &self,
        repository: Uuid,
        location: &StoragePath,
    ) -> Result<Option<crate::StorageFile>, StorageError> {
        match self {
            DynStorage::Local(storage) => storage.open_file(repository, location).await,
        }
    }

    async fn validate_config_change(&self, config: StorageTypeConfig) -> Result<(), StorageError> {
        match self {
            DynStorage::Local(storage) => storage.validate_config_change(config).await,
        }
    }
    async fn put_repository_meta(
        &self,
        repository: Uuid,
        location: &StoragePath,
        value: Value,
    ) -> Result<(), StorageError> {
        match self {
            DynStorage::Local(storage) => {
                storage
                    .put_repository_meta(repository, location, value)
                    .await
            }
        }
    }
    async fn get_repository_meta(
        &self,
        repository: Uuid,
        location: &StoragePath,
    ) -> Result<Option<Value>, StorageError> {
        match self {
            DynStorage::Local(storage) => storage.get_repository_meta(repository, location).await,
        }
    }
}

pub static STORAGE_FACTORIES: &[&dyn StorageFactory] = &[&LocalStorageFactory];

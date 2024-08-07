use derive_more::From;
use uuid::Uuid;

use crate::{
    local::{LocalStorage, LocalStorageFactory},
    FileContent, Storage, StorageError, StorageFactory, StoragePath, StorageTypeConfig,
};
#[derive(Debug, Clone)]
pub enum DynStorage {
    Local(LocalStorage),
}
impl Storage for DynStorage {
    fn unload(&self) -> impl std::future::Future<Output = Result<(), StorageError>> + Send {
        async move {
            match self {
                DynStorage::Local(storage) => storage.unload().await,
            }
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
}

#[derive(Debug, From)]
pub enum DynStorageFactory {
    Local(LocalStorageFactory),
}
impl StorageFactory for DynStorageFactory {
    fn storage_name(&self) -> &'static str {
        match self {
            DynStorageFactory::Local(storage) => storage.storage_name(),
        }
    }

    async fn test_storage_config(
        &self,
        config: crate::StorageTypeConfig,
    ) -> Result<(), StorageError> {
        match self {
            DynStorageFactory::Local(storage) => storage.test_storage_config(config).await,
        }
    }

    async fn create_storage(
        &self,
        config: crate::StorageConfig,
    ) -> Result<DynStorage, StorageError> {
        match self {
            DynStorageFactory::Local(storage) => storage.create_storage(config).await,
        }
    }
}

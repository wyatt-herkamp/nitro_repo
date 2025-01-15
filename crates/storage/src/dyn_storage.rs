use nr_core::storage::StoragePath;
use serde_json::Value;
use uuid::Uuid;

use crate::{
    local::{LocalStorage, LocalStorageFactory},
    meta::RepositoryMeta,
    s3::{S3Storage, S3StorageFactory},
    FileContent, FileType, Storage, StorageError, StorageFactory, StorageTypeConfig,
};
#[derive(Debug, Clone)]
pub enum DynStorage {
    Local(LocalStorage),
    S3(S3Storage),
}
impl Storage for DynStorage {
    type Error = StorageError;
    async fn unload(&self) -> Result<(), StorageError> {
        match self {
            DynStorage::Local(storage) => storage.unload().await.map_err(Into::into),
            DynStorage::S3(storage) => storage.unload().await.map_err(Into::into),
        }
    }

    fn storage_config(&self) -> crate::BorrowedStorageConfig<'_> {
        match self {
            DynStorage::Local(storage) => storage.storage_config(),
            DynStorage::S3(storage) => storage.storage_config(),
        }
    }

    async fn save_file(
        &self,
        repository: Uuid,
        file: FileContent,
        location: &StoragePath,
    ) -> Result<(usize, bool), StorageError> {
        match self {
            DynStorage::Local(storage) => storage
                .save_file(repository, file, location)
                .await
                .map_err(Into::into),
            DynStorage::S3(storage) => storage
                .save_file(repository, file, location)
                .await
                .map_err(Into::into),
        }
    }

    async fn delete_file(
        &self,
        repository: Uuid,
        location: &StoragePath,
    ) -> Result<bool, StorageError> {
        match self {
            DynStorage::Local(storage) => storage
                .delete_file(repository, location)
                .await
                .map_err(Into::into),
            DynStorage::S3(storage) => storage
                .delete_file(repository, location)
                .await
                .map_err(Into::into),
        }
    }

    async fn get_file_information(
        &self,
        repository: Uuid,
        location: &StoragePath,
    ) -> Result<Option<crate::StorageFileMeta<FileType>>, StorageError> {
        match self {
            DynStorage::Local(storage) => storage
                .get_file_information(repository, location)
                .await
                .map_err(Into::into),
            DynStorage::S3(storage) => storage
                .get_file_information(repository, location)
                .await
                .map_err(Into::into),
        }
    }

    async fn open_file(
        &self,
        repository: Uuid,
        location: &StoragePath,
    ) -> Result<Option<crate::StorageFile>, StorageError> {
        match self {
            DynStorage::Local(storage) => storage
                .open_file(repository, location)
                .await
                .map_err(Into::into),
            DynStorage::S3(storage) => storage
                .open_file(repository, location)
                .await
                .map_err(Into::into),
        }
    }

    async fn validate_config_change(&self, config: StorageTypeConfig) -> Result<(), StorageError> {
        match self {
            DynStorage::Local(storage) => storage
                .validate_config_change(config)
                .await
                .map_err(Into::into),
            DynStorage::S3(storage) => storage
                .validate_config_change(config)
                .await
                .map_err(Into::into),
        }
    }
    async fn put_repository_meta(
        &self,
        repository: Uuid,
        location: &StoragePath,
        value: RepositoryMeta,
    ) -> Result<(), StorageError> {
        match self {
            DynStorage::Local(storage) => storage
                .put_repository_meta(repository, location, value)
                .await
                .map_err(Into::into),
            DynStorage::S3(storage) => storage
                .put_repository_meta(repository, location, value)
                .await
                .map_err(Into::into),
        }
    }
    async fn get_repository_meta(
        &self,
        repository: Uuid,
        location: &StoragePath,
    ) -> Result<Option<RepositoryMeta>, StorageError> {
        match self {
            DynStorage::Local(storage) => storage
                .get_repository_meta(repository, location)
                .await
                .map_err(Into::into),
            DynStorage::S3(storage) => storage
                .get_repository_meta(repository, location)
                .await
                .map_err(Into::into),
        }
    }
    async fn file_exists(
        &self,
        repository: Uuid,
        location: &StoragePath,
    ) -> Result<bool, StorageError> {
        match self {
            DynStorage::Local(storage) => storage
                .file_exists(repository, location)
                .await
                .map_err(Into::into),
            DynStorage::S3(storage) => storage
                .file_exists(repository, location)
                .await
                .map_err(Into::into),
        }
    }
}

pub static STORAGE_FACTORIES: &[&dyn StorageFactory] = &[&LocalStorageFactory, &S3StorageFactory];

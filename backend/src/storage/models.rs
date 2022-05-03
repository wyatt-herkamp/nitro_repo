use std::fmt::Debug;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use async_trait::async_trait;

use crate::repository::data::{
    RepositoryConfig, RepositoryDataType, RepositoryMainConfig, RepositorySetting, RepositoryType,
    RepositoryValue,
};
use crate::storage::error::StorageError;
use crate::storage::handler::{StorageHandler, StorageHandlerFactory};

pub static STORAGE_FILE: &str = "storages.json";
pub static STORAGE_FILE_BAK: &str = "storages.json.bak";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    pub public_name: String,
    pub name: String,
    pub created: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnloadedStorage {
    pub storage_handler: StorageHandlerFactory,
    #[serde(flatten)]
    pub config: StorageConfig,
}

#[derive(Debug, Clone)]
pub struct Storage {
    pub storage_handler: StorageHandler,
    pub config: StorageConfig,
}

// Implementation to call the Storage Handler
impl Storage {
    pub async fn create_repository(
        &self,
        name: String,
        repository_type: RepositoryType,
    ) -> Result<RepositoryValue, StorageError> {
        return match &self.storage_handler {
            StorageHandler::LocalStorage(local) => local
                .create_repository(&self.config, name, repository_type)
                .await
                .map_err(StorageError::LocalStorageError),
        };
    }
    pub async fn delete_repository<R: RepositoryDataType>(
        &self,
        repository: &R,
        delete_files: bool,
    ) -> Result<(), StorageError> {
        return match &self.storage_handler {
            StorageHandler::LocalStorage(local) => local
                .delete_repository(&self.config, repository, delete_files)
                .await
                .map_err(StorageError::LocalStorageError),
        };
    }
    pub async fn get_repositories(&self) -> Result<Vec<RepositoryValue>, StorageError> {
        return match &self.storage_handler {
            StorageHandler::LocalStorage(local) => local
                .get_repositories(&self.config)
                .await
                .map_err(StorageError::LocalStorageError),
        };
    }
    pub async fn get_repository<T: RepositorySetting>(
        &self,
        name: &str,
    ) -> Result<Option<RepositoryConfig<T>>, StorageError> {
        return match &self.storage_handler {
            StorageHandler::LocalStorage(local) => local
                .get_repository(&self.config, name)
                .await
                .map_err(StorageError::LocalStorageError),
        };
    }
    pub async fn get_repository_value(
        &self,
        name: &str,
    ) -> Result<Option<RepositoryValue>, StorageError> {
        return match &self.storage_handler {
            StorageHandler::LocalStorage(local) => local
                .get_repository_value(&self.config, name)
                .await
                .map_err(StorageError::LocalStorageError),
        };
    }
    pub async fn update_repository<T: RepositorySetting>(
        &self,
        repository: RepositoryMainConfig<T>,
    ) -> Result<(), StorageError> {
        return match &self.storage_handler {
            StorageHandler::LocalStorage(local) => local
                .update_repository(&self.config, repository)
                .await
                .map_err(StorageError::LocalStorageError),
        };
    }
    //File Handlers
    pub async fn save_file<R: RepositoryDataType>(
        &self,
        repository: &R,
        file: &[u8],
        location: &str,
    ) -> Result<(), StorageError> {
        return match &self.storage_handler {
            StorageHandler::LocalStorage(local) => local
                .save_file(&self.config, repository, file, location)
                .await
                .map_err(StorageError::LocalStorageError),
        };
    }
    pub async fn delete_file<R: RepositoryDataType>(
        &self,
        repository: &R,
        location: &str,
    ) -> Result<(), StorageError> {
        return match &self.storage_handler {
            StorageHandler::LocalStorage(local) => local
                .delete_file(&self.config, repository, location)
                .await
                .map_err(StorageError::LocalStorageError),
        };
    }
    pub async fn get_file_as_response<R: RepositoryDataType>(
        &self,
        repository: &R,
        location: &str,
    ) -> Result<Option<StorageFileResponse>, StorageError> {
        return match &self.storage_handler {
            StorageHandler::LocalStorage(local) => local
                .get_file_as_response(&self.config, repository, location)
                .await
                .map_err(StorageError::LocalStorageError),
        };
    }
    pub async fn get_file<R: RepositoryDataType>(
        &self,
        repository: &R,
        location: &str,
    ) -> Result<Option<Vec<u8>>, StorageError> {
        return match &self.storage_handler {
            StorageHandler::LocalStorage(local) => local
                .get_file(&self.config, repository, location)
                .await
                .map_err(StorageError::LocalStorageError),
        };
    }
}

///Storage Files are just a data container holding the file name, directory relative to the root of nitro_repo and if its a directory
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct StorageFile {
    pub name: String,
    pub full_path: String,
    pub directory: bool,
    pub file_size: u64,
    pub created: u128,
}

/// StorageFileResponse is a trait that can be turned into a SiteResponse for example if its  LocalFile it will return into Actix's NamedFile response
pub enum StorageFileResponse {
    File(PathBuf),
    List(Vec<StorageFile>),
    Bytes,
}

#[async_trait]
pub trait StorageType {
    type Error;
    type StorageConfig;
    /// Initialize the Storage at Storage start.
    async fn load(&mut self, config: &StorageConfig) -> Result<(), Self::Error>;
    /// Unload the storage
    fn unload(&mut self) -> Result<(), Self::Error>;

    /// Creates a new Repository
    async fn create_repository(
        &self,
        config: &StorageConfig,
        name: String,
        repository_type: RepositoryType,
    ) -> Result<RepositoryValue, Self::Error>;
    /// Deletes a Repository
    async fn delete_repository<R: RepositoryDataType>(
        &self,
        config: &StorageConfig,
        repository: &R,
        delete_files: bool,
    ) -> Result<(), Self::Error>;
    /// Gets a Vec of Repository Values
    async fn get_repositories(
        &self,
        config: &StorageConfig,
    ) -> Result<Vec<RepositoryValue>, Self::Error>;
    async fn get_repository_value(
        &self,
        config: &StorageConfig,
        repository: &str,
    ) -> Result<Option<RepositoryValue>, Self::Error>;
    /// Gets the RepositoryMainConfig
    async fn get_repository<RS: RepositorySetting>(
        &self,
        config: &StorageConfig,
        uuid: &str,
    ) -> Result<Option<RepositoryConfig<RS>>, Self::Error>;
    async fn update_repository<RS: RepositorySetting>(
        &self,
        config: &StorageConfig,
        repository: RepositoryMainConfig<RS>,
    ) -> Result<(), Self::Error>;
    //File Handlers
    async fn save_file<R: RepositoryDataType>(
        &self,
        config: &StorageConfig,
        repository: &R,
        file: &[u8],
        location: &str,
    ) -> Result<(), Self::Error>;
    async fn delete_file<R: RepositoryDataType>(
        &self,
        config: &StorageConfig,
        repository: &R,
        location: &str,
    ) -> Result<(), Self::Error>;
    async fn get_file_as_response<R: RepositoryDataType>(
        &self,
        config: &StorageConfig,
        repository: &R,
        location: &str,
    ) -> Result<Option<StorageFileResponse>, Self::Error>;
    async fn get_file<R: RepositoryDataType>(
        &self,
        config: &StorageConfig,
        repository: &R,
        location: &str,
    ) -> Result<Option<Vec<u8>>, Self::Error>;
}

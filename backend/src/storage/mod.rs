use async_trait::async_trait;
use bytes::Bytes;
use futures::Stream;
use lockfree::map::Removed;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::RwLockReadGuard;

use crate::repository::settings::{RepositoryConfig, RepositoryType};
use crate::storage::error::StorageError;
use crate::storage::file::{StorageFile, StorageFileResponse};
use crate::storage::models::{Storage, StorageStatus};

pub mod bad_storage;
pub mod error;
pub mod file;
pub mod local_storage;
pub mod models;
pub mod multi;
pub static STORAGES_CONFIG: &str = "storages.nitro_repo";
pub static STORAGE_CONFIG: &str = "storage.nitro_repo";
use crate::repository::handler::DynamicRepositoryHandler;
use bad_storage::BadStorage;
use local_storage::{LocalConfig, LocalStorage};
use std::collections::HashMap;
use std::hint::unreachable_unchecked;
macro_rules! storages {
    ($($name:ident, $ty:tt, $config:tt),*) => {
        #[derive(Debug, Clone, Serialize, Deserialize)]
        pub enum StorageType{
            $($name,)*
        }
        #[derive(Debug, Clone, Serialize, Deserialize)]
        pub enum StorageConfig {
            $($name($config),)*
        }
        #[derive(Debug)]
        pub enum DynamicStorage{
            BadStorage(BadStorage),
            $($name($ty),)*
        }
#[async_trait]
impl Storage for DynamicStorage{
    type Repository = DynamicRepositoryHandler<DynamicStorage>;

    async fn create_new(config: StorageSaver) -> Result<Self, (StorageError, StorageSaver)> where Self: Sized {
        match config.storage_type.clone() {
            $(StorageType::$name => {
                let factory = config.clone();
                $ty::create_new(config).await.map(|storage| DynamicStorage::$name(storage))
            })*
        }
    }

    async fn new(config: StorageSaver) -> Result<Self, (StorageError, StorageSaver)> where Self: Sized {
        match config.storage_type.clone() {
            $(StorageType::$name => {
                let factory = config.clone();
                $ty::new(config).await.map(|storage| DynamicStorage::$name(storage))
            })*
        }
    }
    async fn get_repos_to_load(&self) -> Result<HashMap<String, RepositoryConfig>, StorageError>{
        match self {
            $(DynamicStorage::$name(storage) => storage.get_repos_to_load().await,)*
            _ => unsafe{ unreachable_unchecked() }
        }
    }

    fn add_repo_loaded<R: Into<Self::Repository> + Send>(&self, repo: R) -> Result<(), StorageError>{
        match self {
            $(DynamicStorage::$name(storage) => storage.add_repo_loaded(repo),)*
            _ => unsafe{ unreachable_unchecked() }
        }
    }

    fn unload(&mut self) -> Result<(), StorageError> {
        match self {
            $(DynamicStorage::$name(storage) => storage.unload(),)*
            _ => unsafe{ unreachable_unchecked() }
        }
    }

    fn storage_config(&self) -> &StorageSaver {
        match self {
            $(DynamicStorage::$name(storage) => storage.storage_config(),)*
            DynamicStorage::BadStorage(storage) => &storage.factory,
        }
    }

    async fn create_repository<R: Into<Self::Repository> + Send>(&self, repository: R) -> Result<(), StorageError> {
        todo!()
    }

    async fn delete_repository<S: AsRef<str> + Send>(&self, repository: S, delete_files: bool) -> Result<(), StorageError> {
        todo!()
    }

    fn get_repository_list(&self) -> Result<Vec<RepositoryConfig>, StorageError> {
        todo!()
    }

    fn get_repository<S: AsRef<str>>(&self, repository: S) -> Result<Option<Arc<Self::Repository>>, StorageError> {
        todo!()
    }

    fn remove_repository_for_updating<S: AsRef<str>>(&self, repository: S) -> Result<Removed<String, Arc<Self::Repository>>, StorageError> {
        todo!()
    }

    async fn add_repository_for_updating(&self, repository_arc: Self::Repository) -> Result<(), StorageError> {
        todo!()
    }

    async fn add_repository_for_updating_removed(&self, repository_arc: Removed<String, Arc<Self::Repository>>) -> Result<(), StorageError> {
        todo!()
    }

    async fn save_file(&self, repository: &RepositoryConfig, file: &[u8], location: &str) -> Result<bool, StorageError> {
        todo!()
    }

    fn write_file_stream<S: Stream<Item=Bytes> + Unpin + Send + Sync + 'static>(&self, repository: &RepositoryConfig, s: S, location: &str) -> Result<bool, StorageError> {
        todo!()
    }

    async fn delete_file(&self, repository: &RepositoryConfig, location: &str) -> Result<(), StorageError> {
        todo!()
    }

    async fn get_file_as_response(&self, repository: &RepositoryConfig, location: &str) -> Result<StorageFileResponse, StorageError> {
        todo!()
    }

    async fn get_file_information(&self, repository: &RepositoryConfig, location: &str) -> Result<Option<StorageFile>, StorageError> {
        todo!()
    }

    async fn get_file(&self, repository: &RepositoryConfig, location: &str) -> Result<Option<Vec<u8>>, StorageError> {
        todo!()
    }
    async fn get_repository_config<ConfigType: DeserializeOwned>(
        &self,
        repository: &RepositoryConfig,
        config_name: &str,
    ) -> Result<Option<ConfigType>, StorageError> {
        match self {
            $(DynamicStorage::$name(storage) => storage.get_repository_config(repository, config_name).await,)*
            _ => unsafe{ unreachable_unchecked() }
        }
    }
}
    };
}

storages!(LocalStorage, LocalStorage, LocalConfig);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageSaver {
    /// The Type of the Storage
    pub storage_type: StorageType,
    /// The Storage Config
    #[serde(flatten)]
    pub generic_config: GeneralConfig,
    /// Storage Handler Config
    pub handler_config: StorageConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    pub id: String,
    /// This is created internally by the storage. No need to set this.
    #[serde(default = "crate::utils::get_current_time")]
    pub created: i64,
}

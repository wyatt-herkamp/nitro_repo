use async_trait::async_trait;
use bytes::Bytes;
use futures::Stream;
use lock_freedom::map::Removed;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use std::sync::Arc;

use crate::repository::settings::RepositoryConfig;
use crate::storage::error::StorageError;
use crate::storage::file::{StorageFile, StorageFileResponse};
use crate::storage::models::Storage;

pub mod bad_storage;
pub mod error;
pub mod file;
pub mod local_storage;
pub mod models;
pub mod multi;
pub mod path;

use path::StoragePath;
use path::SystemStorageFile;

pub const STORAGES_CONFIG: &'static str = "storages.json";
pub const STORAGE_CONFIG: &'static str = "storage.json";
pub const REPOSITORY_FOLDER: &'static str = ".config.nitro_repo";

use crate::repository::handler::DynamicRepositoryHandler;
use bad_storage::BadStorage;
use chrono::{DateTime, FixedOffset};
use local_storage::{LocalConfig, LocalStorage};
use std::collections::HashMap;
use std::hint::unreachable_unchecked;
macro_rules! storages {
    ($($name:ident, $ty:tt, $config:tt),*) => {
        #[derive(Debug, Clone, Serialize, Deserialize)]
        #[serde(tag = "storage_type", content = "settings")]
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
        match config.handler_config {
            $(StorageConfig::$name(_) => {
                let _factory = config.clone();
                $ty::create_new(config).await.map(|storage| DynamicStorage::$name(storage))
            })*
        }
    }

    async fn new(config: StorageSaver) -> Result<Self, (StorageError, StorageSaver)> where Self: Sized {
        match config.handler_config {
            $(StorageConfig::$name(_) => {
                let _factory = config.clone();
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

    async fn create_repository<R: Into<Self::Repository> + Send>(&self, repository: R) -> Result<Arc<Self::Repository>, StorageError> {
        match self {
            $(DynamicStorage::$name(storage) => storage.create_repository(repository).await,)*
            _ => unsafe{ unreachable_unchecked() }
        }
    }

    async fn delete_repository<S: AsRef<str> + Send>(&self, repository: S, delete_files: bool) -> Result<(), StorageError> {
        match self {
            $(DynamicStorage::$name(storage) => storage.delete_repository(repository,delete_files).await,)*
            _ => unsafe{ unreachable_unchecked() }
        }
    }

    fn get_repository_list(&self) -> Result<Vec<RepositoryConfig>, StorageError> {
        match self {
            $(DynamicStorage::$name(storage) => storage.get_repository_list(),)*
            _ => unsafe{ unreachable_unchecked() }
        }
    }

    fn get_repository<S: AsRef<str>>(&self, repository: S) -> Result<Option<Arc<Self::Repository>>, StorageError> {
        match self {
            $(DynamicStorage::$name(storage) => storage.get_repository(repository),)*
            _ => unsafe{ unreachable_unchecked() }
        }
    }

    fn remove_repository_for_updating<S: AsRef<str>>(&self, repository: S) -> Option<Removed<String, Arc<Self::Repository>>> {
        match self {
            $(DynamicStorage::$name(storage) => storage.remove_repository_for_updating(repository),)*
            _ => unsafe{ unreachable_unchecked() }
        }
    }

    async fn add_repository_for_updating(&self,name: String, repository_arc: Self::Repository, save: bool) -> Result<(), StorageError> {
        match self {
            $(DynamicStorage::$name(storage) => storage.add_repository_for_updating(name, repository_arc,save).await,)*
            _ => unsafe{ unreachable_unchecked() }
        }
    }

    async fn save_file(&self, repository: &RepositoryConfig, file: &[u8], location: &str) -> Result<bool, StorageError> {
        match self {
            $(DynamicStorage::$name(storage) => storage.save_file(repository,file,location).await,)*
            _ => unsafe{ unreachable_unchecked() }
        }
    }

    fn write_file_stream<S: Stream<Item=Bytes> + Unpin + Send + Sync + 'static>(&self, repository: &RepositoryConfig, s: S, location: &str) -> Result<bool, StorageError> {
        match self {
            $(DynamicStorage::$name(storage) => storage.write_file_stream(repository,s,location),)*
            _ => unsafe{ unreachable_unchecked() }
        }
    }

    async fn delete_file(&self, repository: &RepositoryConfig, location: &str) -> Result<(), StorageError> {
        match self {
            $(DynamicStorage::$name(storage) => storage.delete_file(repository,location).await,)*
            _ => unsafe{ unreachable_unchecked() }
        }
    }

    async fn get_file_as_response(&self, repository: &RepositoryConfig, location: &str) -> Result<StorageFileResponse, StorageError> {
        match self {
            $(DynamicStorage::$name(storage) => storage.get_file_as_response(repository,location).await,)*
            _ => unsafe{ unreachable_unchecked() }
        }
    }

    async fn get_file_information(&self, repository: &RepositoryConfig, location: &str) -> Result<Option<StorageFile>, StorageError> {
        match self {
            $(DynamicStorage::$name(storage) => storage.get_file_information(repository,location).await,)*
            _ => unsafe{ unreachable_unchecked() }
        }
    }

    async fn get_file(&self, repository: &RepositoryConfig, location: &str) -> Result<Option<Vec<u8>>, StorageError> {
        match self {
            $(DynamicStorage::$name(storage) => storage.get_file(repository,location).await,)*
            _ => unsafe{ unreachable_unchecked() }
        }
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
    async fn save_repository_config<ConfigType: crate::repository::settings::RepositoryConfigType>(
        &self,
        repository: &RepositoryConfig,
        config: &ConfigType,
    ) -> Result<(), StorageError> {
        match self {
            $(DynamicStorage::$name(storage) => storage.save_repository_config(repository, config).await,)*
            _ => unsafe{ unreachable_unchecked() }
        }
    }
    async fn list_files<S: AsRef<str>+ Send, SP: Into<StoragePath>+ Send>(
        &self,
        repository: S,
        path: SP,
    ) -> Result<Vec<SystemStorageFile>, StorageError>{
        match self {
            $(DynamicStorage::$name(storage) => storage.list_files(repository, path).await,)*
            _ => unsafe{ unreachable_unchecked() }
        }
    }
}
    };
}

storages!(LocalStorage, LocalStorage, LocalConfig);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageSaver {
    /// The Storage Config
    #[serde(flatten)]
    pub generic_config: GeneralConfig,
    /// Storage Handler Config
    pub handler_config: StorageConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    pub id: String,
    /// This is created internally by the storages. No need to set this.
    #[serde(
        deserialize_with = "crate::time_fix::read_time",
        default = "crate::utils::get_current_date_time_struct"
    )]
    pub created: DateTime<FixedOffset>,
}

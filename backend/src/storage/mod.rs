use std::{
    collections::HashMap,
    fmt::{Debug, Display, Formatter},
    sync::Arc,
};

use bytes::Bytes;
use futures::Stream;
use lock_freedom::map::Removed;
use nitro_repo_internal_macros::dynamic_storage;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::{
    repository::{
        handler::Repository,
        settings::{RepositoryConfig, RepositoryConfigType},
    },
    storage::{
        error::StorageError,
        file::{StorageFile, StorageFileResponse},
        path::{StoragePath, SystemStorageFile},
    },
};
pub mod bad_storage;
pub mod error;
pub mod file;
pub mod local_storage;
pub mod multi;
pub mod path;

pub const STORAGES_CONFIG: &'static str = "storages.nitro_repo";
pub const STORAGE_CONFIG: &'static str = "storage.nitro_repo";
pub const REPOSITORY_FOLDER: &'static str = ".config.nitro_repo";

use bad_storage::BadStorage;
use chrono::{DateTime, FixedOffset};
use local_storage::{LocalConfig, LocalStorage};

use crate::repository::handler::DynamicRepositoryHandler;

pub static STORAGE_FILE: &str = "storages.json";
pub static STORAGE_FILE_BAK: &str = "storages.json.bak";

/// Storage Status
#[derive(Debug)]
pub enum StorageStatus {
    /// The storages is unloaded.
    Unloaded,
    /// Storage has successfully loaded
    Loaded,
    /// Storage Errored out during loading
    Error(StorageError),
    /// Storage Errored out during creation. Usually meaning bad config
    CreateError(StorageError),
}

impl PartialEq for StorageStatus {
    fn eq(&self, other: &Self) -> bool {
        self.to_string() == other.to_string()
    }
}

impl Display for StorageStatus {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            StorageStatus::Unloaded => {
                write!(f, "Unloaded")
            }
            StorageStatus::Loaded => {
                write!(f, "Loaded")
            }
            StorageStatus::Error(error) => {
                write!(f, "Error while loading: {}", error)
            }
            StorageStatus::CreateError(error) => {
                write!(f, "Error While Creating: {}", error)
            }
        }
    }
}
pub trait Storage: Send + Sync {
    type Repository: Repository<DynamicStorage>;
    /// Initialize the Storage at Storage start.
    async fn create_new(config: StorageSaver) -> Result<Self, (StorageError, StorageSaver)>
    where
        Self: Sized;
    /// Initialize the Storage at Storage start.
    async fn new(config: StorageSaver) -> Result<Self, (StorageError, StorageSaver)>
    where
        Self: Sized;
    // Attempts to Load the Storage
    async fn get_repos_to_load(&self) -> Result<HashMap<String, RepositoryConfig>, StorageError>;

    fn add_repo_loaded<R: Into<Self::Repository> + Send>(
        &self,
        repo: R,
    ) -> Result<(), StorageError>;

    /// Unload the storages
    fn unload(&mut self) -> Result<(), StorageError>;

    fn storage_config(&self) -> &StorageSaver;
    /// Creates a new Repository
    /// Requires just the name and RepositoryType
    async fn create_repository<R: Into<Self::Repository> + Send>(
        &self,
        repository: R,
    ) -> Result<Arc<Self::Repository>, StorageError>;
    /// Deletes a Repository
    /// delete_files rather or not to clean out the Repository Data
    async fn delete_repository<S: AsRef<str> + Send>(
        &self,
        repository: S,
        delete_files: bool,
    ) -> Result<(), StorageError>;
    /// Gets a Owned Vec of Repositories
    fn get_repository_list(&self) -> Result<Vec<RepositoryConfig>, StorageError>;
    /// Returns a RwLockReadGuard of the RepositoryConfig
    fn get_repository<S: AsRef<str>>(
        &self,
        repository: S,
    ) -> Result<Option<Arc<Self::Repository>>, StorageError>;

    fn remove_repository_for_updating(
        &self,
        repository: &str,
    ) -> Option<Removed<String, Arc<Self::Repository>>>;
    /// Will update all configs for the Repository
    async fn add_repository_for_updating(
        &self,
        name: String,
        repository_arc: Self::Repository,
        save: bool,
    ) -> Result<(), StorageError>;
    /// Saves a File to a location
    /// Will overwrite any data found
    async fn save_file(
        &self,
        repository: &RepositoryConfig,
        file: &[u8],
        location: &str,
    ) -> Result<bool, StorageError>;
    fn write_file_stream<S: Stream<Item = Bytes> + Unpin + Send + Sync + 'static>(
        &self,
        repository: &RepositoryConfig,
        s: S,
        location: &str,
    ) -> Result<bool, StorageError>;

    /// Deletes a file at a given location
    async fn delete_file(
        &self,
        repository: &RepositoryConfig,
        location: &str,
    ) -> Result<(), StorageError>;
    /// Gets tje File as a StorageFileResponse
    /// Can be converted for Web Responses
    async fn get_file_as_response(
        &self,
        repository: &RepositoryConfig,
        location: &str,
    ) -> Result<StorageFileResponse, StorageError>;
    /// Returns Information about the file
    async fn get_file_information(
        &self,
        repository: &RepositoryConfig,
        location: &str,
    ) -> Result<Option<StorageFile>, StorageError>;
    /// Gets the File as an Array of Bytes
    /// Used for internal processing
    async fn get_file(
        &self,
        repository: &RepositoryConfig,
        location: &str,
    ) -> Result<Option<Vec<u8>>, StorageError>;
    /// Gets a Repository Config
    async fn get_repository_config<ConfigType: DeserializeOwned>(
        &self,
        repository: &RepositoryConfig,
        config_name: &str,
    ) -> Result<Option<ConfigType>, StorageError>;

    async fn save_repository_config<ConfigType: RepositoryConfigType>(
        &self,
        repository: &RepositoryConfig,
        config: &ConfigType,
    ) -> Result<(), StorageError>;

    async fn list_files<SP: Into<StoragePath> + Send>(
        &self,
        repository: &str,
        path: SP,
    ) -> Result<Vec<SystemStorageFile>, StorageError>;
}
#[dynamic_storage]
pub enum DynamicStorage {
    LocalStorage {
        storage: LocalStorage,
        config: LocalConfig,
    },
}

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

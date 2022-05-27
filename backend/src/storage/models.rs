use std::fmt::{Debug, Display, Formatter};

use serde::{Deserialize, Serialize};

use async_trait::async_trait;

use serde_json::Value;
use tokio::sync::RwLockReadGuard;

use crate::repository::data::{RepositoryConfig, RepositoryType};
use crate::storage::error::StorageError;
use crate::storage::file::{StorageFile, StorageFileResponse};
use crate::storage::local_storage::LocalStorage;
use crate::storage::DynamicStorage;

pub static STORAGE_FILE: &str = "storages.json";
pub static STORAGE_FILE_BAK: &str = "storages.json.bak";

/// Types of Storages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StorageType {
    LocalStorage,
}

/// Storage Status
#[derive(Debug)]
pub enum StorageStatus {
    /// The storage is unloaded.
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
                write!(f, "{}", error)
            }
            StorageStatus::CreateError(error) => {
                write!(f, "{}", error)
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageSaver {
    pub storage_type: StorageType,
    #[serde(flatten)]
    pub generic_config: StorageConfig,
    /// Storage Handler Config
    pub handler_config: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    pub public_name: String,
    pub name: String,
    pub created: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageFactory {
    pub storage_type: StorageType,
    #[serde(flatten)]
    pub generic_config: StorageConfig,
    /// Storage Handler Config
    pub handler_config: Value,
}

impl StorageFactory {
    pub fn config_for_saving(&self) -> StorageSaver {
        StorageSaver {
            storage_type: self.storage_type.clone(),
            generic_config: self.generic_config.clone(),
            handler_config: self.handler_config.clone(),
        }
    }
}

impl StorageFactory {
    pub async fn build(self) -> Result<DynamicStorage, (StorageError, StorageFactory)> {
        match &self.storage_type {
            StorageType::LocalStorage => {
                LocalStorage::new(self).map(|v| DynamicStorage::LocalStorage(v))
            }
        }
    }
}

#[async_trait]
pub trait Storage: Send + Sync {
    /// Initialize the Storage at Storage start.
    fn new(config: StorageFactory) -> Result<Self, (StorageError, StorageFactory)>
    where
        Self: Sized;
    // Attempts to Load the Storage
    async fn load(&mut self) -> Result<(), StorageError>;
    /// Unload the storage
    fn unload(&mut self) -> Result<(), StorageError>;
    /// Returns a StorageSaver
    /// I would like this to be a data reference in the future
    fn config_for_saving(&self) -> StorageSaver;

    fn storage_config(&self) -> &StorageConfig;
    /// Returns a Owned copy of the Implementation of Type Config. As a serde_json::Value
    fn impl_config(&self) -> Value;
    /// Returns the Storage Type
    fn storage_type(&self) -> &StorageType;
    /// The current status of the Storage
    fn status(&self) -> &StorageStatus;
    /// Creates a new Repository
    /// Requires just the name and RepositoryType
    async fn create_repository(
        &self,
        name: String,
        repository_type: RepositoryType,
    ) -> Result<(), StorageError>;
    /// Deletes a Repository
    /// delete_files rather or not to clean out the Repository Data
    async fn delete_repository(
        &self,
        repository: &RepositoryConfig,
        delete_files: bool,
    ) -> Result<(), StorageError>;
    /// Gets a Owned Vec of Repositories
    async fn get_repositories(&self) -> Result<Vec<RepositoryConfig>, StorageError>;
    /// Returns a RwLockReadGuard of the RepositoryConfig
    async fn get_repository(
        &self,
        repository: &str,
    ) -> Result<Option<RwLockReadGuard<'_, RepositoryConfig>>, StorageError>;

    async fn update_repository(&self, repository: RepositoryConfig) -> Result<(), StorageError>;
    /// Locks the Repositories for updating
    /// Keeps all config files in .config
    async fn update_repository_config(
        &self,
        repository: &RepositoryConfig,
        file: &str,
        data: &Option<Value>,
    ) -> Result<(), StorageError>;
    /// Gets a Repository Config
    async fn get_repository_config(
        &self,
        repository: &RepositoryConfig,
        file: &str,
    ) -> Result<Option<Value>, StorageError>;

    /// Saves a File to a location
    /// Will overwrite any data found
    async fn save_file(
        &self,
        repository: &RepositoryConfig,
        file: &[u8],
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
}

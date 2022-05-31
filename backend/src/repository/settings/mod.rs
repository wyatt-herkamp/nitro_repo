use std::fmt::Debug;

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::error::internal_error::InternalError;
use crate::repository::settings::security::Visibility;
use crate::storage::models::Storage;

pub mod frontend;
pub mod security;
pub mod webhook;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, strum_macros::EnumString)]
pub enum Policy {
    Release,
    Snapshot,
    Mixed,
}

impl Default for Policy {
    fn default() -> Self {
        Policy::Mixed
    }
}

fn default() -> bool {
    true
}

/// Types of Repositories that can exist.
#[derive(Serialize, Deserialize, Clone, Debug, strum_macros::Display, strum_macros::EnumString)]
pub enum RepositoryType {
    /// A Maven Repository
    Maven,
    /// A NPM Repository
    NPM,
}

/// The Basic Repository Config
/// These values are core to the existence of the Repository
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RepositoryConfig {
    /// Repository Name
    pub name: String,
    /// The Type of Repository
    pub repository_type: RepositoryType,
    /// Storage.
    pub storage: String,
    /// Visibility.
    #[serde(default)]
    pub visibility: Visibility,
    /// Rather or not the Repository is active. Meaning it can be pulled or pushed
    #[serde(default = "default")]
    pub active: bool,
    ///The versioning policy for the Repository
    #[serde(default)]
    pub policy: Policy,
    /// When it was created
    pub created: i64,
}

impl RepositoryConfig {
    /// Gets the Config File
    /// # Arguments
    /// * `storage` - The Storage to use. The Repository must belong to the Storage
    /// # Returns
    /// * `Result<Option<T>, InternalError>` - The Config File
    pub async fn get_config<Config: RepositoryConfigType, StorageType: Storage>(
        &self,
        storage: &StorageType,
    ) -> Result<Option<Config>, InternalError> {
        storage
            .get_repository_config::<Config>(self, Config::config_name())
            .await
            .map_err(InternalError::StorageError)
    }
    /// Saves the repository config
    /// # Arguments
    /// * `storage` - The Storage to use. The Repository must belong to the Storage
    /// * `config` - The Config to save. If None is passed, then the config is deleted
    /// # Returns
    /// * `Result<(), InternalError>` -
    /// # Errors
    /// * `InternalError::StorageError` - If the Storage fails to save the config
    /// # Remarks
    /// This will overwrite the config file.
    pub async fn save_config<Config: RepositoryConfigType, StorageType: Storage>(
        &self,
        storage: &StorageType,
        config: Option<&Config>,
    ) -> Result<(), InternalError> {
        if let Some(value) = config {
            storage
                .update_repository_config(self, Config::config_name(), &value)
                .await
                .map_err(InternalError::StorageError)
        } else {
            // Deleting the config means for the Config type that the feature set has been disabled
            // If the type should not be disabled. This is a bug.
            storage
                .delete_repository_config(self, Config::config_name())
                .await
                .map_err(InternalError::StorageError)
        }
    }
}

/// Represents a Repository Setting group. That is not apart of the core config set
pub trait RepositoryConfigType: Send + Sync + Clone + Debug + Serialize + DeserializeOwned {
    fn config_name() -> &'static str;
}

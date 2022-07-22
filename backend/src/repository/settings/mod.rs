use std::fmt::Debug;

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::error::internal_error::InternalError;
pub use crate::repository::handler::RepositoryType;
use crate::storage::models::Storage;

pub mod badge;
pub mod frontend;
pub mod post_deploy;
pub mod repository_page;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, strum_macros::EnumString)]
pub enum Visibility {
    Public,
    Private,
    Hidden,
}

impl Default for Visibility {
    fn default() -> Self {
        Visibility::Public
    }
}

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
        _storage: &StorageType,
        _config: Option<&Config>,
    ) -> Result<(), InternalError> {
        panic!("To be replaced");
    }
}

/// Represents a Repository Setting group. That is not apart of the core config set
pub trait RepositoryConfigType: Send + Sync + Clone + Debug + Serialize + DeserializeOwned {
    fn config_name() -> &'static str;
    fn from_slice_json(slice: &[u8]) -> Result<Self, serde_json::Error> {
        serde_json::from_slice(slice)
    }
}

pub trait RepositoryConfigHandler<Config: RepositoryConfigType> {
    #[inline(always)]
    fn supports_config(&self) -> bool {
        true
    }

    fn update(&mut self, config: Config) -> Result<(), InternalError>;

    fn get(&self) -> &Config;

    fn get_mut(&mut self) -> &mut Config;
}
macro_rules! define_config_handler {
    ($name:ident, $handler:ty, $config:ty, $check:ident) => {
        impl<StorageType: Storage> crate::repository::settings::RepositoryConfigHandler<$config>
            for $handler
        {
            fn update(&mut self, mut config: $config) -> Result<(), InternalError> {
                self.$name = $check(config)?;
                Ok(())
            }
            fn get(&self) -> &$config {
                &self.$name
            }
            fn get_mut(&mut self) -> &mut $config {
                &mut self.$name
            }
        }
    };
    ($name:ident, $handler:ty, $config:ident) => {
        impl<StorageType: Storage> crate::repository::settings::RepositoryConfigHandler<$config>
            for $handler
        {
            fn update(&mut self, mut config: $config) -> Result<(), InternalError> {
                self.$name = config;
                Ok(())
            }
            fn get(&self) -> &$config {
                &self.$name
            }
            fn get_mut(&mut self) -> &mut $config {
                &mut self.$name
            }
        }
    };
}
pub(crate) use define_config_handler;

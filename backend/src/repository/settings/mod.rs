use schemars::schema::RootSchema;
use schemars::JsonSchema;

use std::fmt::Debug;
use chrono::{DateTime, FixedOffset, Local};

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::error::internal_error::InternalError;
pub use crate::repository::handler::RepositoryType;
use crate::storage::models::Storage;

pub mod badge;
pub mod frontend;
pub mod repository_page;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum Visibility {
    #[default]
    Public,
    Private,
    Hidden,
}

#[derive(
    Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default, JsonSchema, strum_macros::Display,
)]
pub enum Policy {
    #[default]
    Release,
    Snapshot,
    Mixed,
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
    pub active: bool,
    #[serde(default)]
    pub require_token_over_basic: bool,

    #[serde(deserialize_with  = "crate::time_fix::read_time")]
    pub created: DateTime<FixedOffset>,
}

impl RepositoryConfigType for RepositoryConfig {
    fn config_name() -> &'static str {
        "config.json"
    }
}

impl RepositoryConfig {
    pub fn new(name: impl Into<String>, repository_type: RepositoryType, storage: String) -> Self {
        RepositoryConfig {
            name: name.into(),
            repository_type,
            storage,
            visibility: Visibility::default(),
            active: true,
            require_token_over_basic: false,
            created: Local::now().into(),
        }
    }
}

impl AsRef<str> for RepositoryConfig {
    fn as_ref(&self) -> &str {
        &self.name
    }
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

#[derive(Serialize, Debug)]
pub struct RepositoryLayoutValue {
    pub config_name: &'static str,
    pub config_proper_name: &'static str,
    pub schema: RootSchema,
}

pub trait RepositoryConfigLayout {
    fn get_config_layout(&self) -> Vec<RepositoryLayoutValue>;
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
    ($name:ident, $handler:ty, $config:ty, $update:ident) => {
        impl<StorageType: Storage> crate::repository::settings::RepositoryConfigHandler<$config>
            for $handler
        {
            fn update(&mut self, mut config: $config) -> Result<(), InternalError> {
                self.$update()
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
            fn update(&mut self, config: $config) -> Result<(), InternalError> {
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
macro_rules! define_config_layout {
    ($handler:ty, $($name:ident, $config:ident),*) => {
        impl<StorageType: Storage> crate::repository::settings::RepositoryConfigLayout for $handler {
            fn get_config_layout(&self) -> Vec<crate::repository::settings::RepositoryLayoutValue> {
                let mut layout = Vec::new();
                $(
                    layout.push(crate::repository::settings::RepositoryLayoutValue{
                        config_name: stringify!($name),
                        config_proper_name: stringify!($name),
                        schema: schemars::schema_for!($config)
                    });
                )*
                layout
            }
        }
    };
    ($handler:ty, $($name:ident, $proper_name:literal, $config:ident),*) => {
        impl<StorageType: Storage> crate::repository::settings::RepositoryConfigLayout for $handler {
            fn get_config_layout(&self) -> Vec<crate::repository::settings::RepositoryLayoutValue> {
                let mut layout = Vec::new();
                $(
                    layout.push(crate::repository::settings::RepositoryLayoutValue{
                        config_name: stringify!($name),
                        config_proper_name:  $proper_name,
                        schema: schemars::schema_for!($config)
                    });
                )*
                layout
            }
        }
    };
    ($handler:ty)=>{
        impl<StorageType: Storage> crate::repository::settings::RepositoryConfigLayout for $handler {
            fn get_config_layout(&self) -> Vec<crate::repository::settings::RepositoryLayoutValue> {
                return Vec::new();
            }
        }
    }
}
macro_rules! define_configs_on_handler {
    ($handler:ty) => {
        crate::repository::settings::define_config_layout!($handler);
    };
    ($handler:ty, $($name:ident, $config:ident),*) => {
        $(
            crate::repository::settings::define_config_handler!($name, $handler, $config);
        )*
        crate::repository::settings::define_config_layout!($handler, $($name, $config),*);
    };
}

use crate::utils::get_current_time;
pub(crate) use define_config_handler;
pub(crate) use define_config_layout;
pub(crate) use define_configs_on_handler;

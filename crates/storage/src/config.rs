use chrono::Utc;
use nr_core::{database::storage::DBStorage, ConfigTimeStamp};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{local::LocalConfig, s3::S3Config, StorageError};
#[derive(Debug, Clone, Error)]
#[error("Expected Config Type: {0}, Got: {1}")]
pub struct InvalidConfigType(&'static str, &'static str);
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct StorageConfigInner {
    pub storage_name: String,
    pub storage_id: Uuid,
    pub storage_type: String,
    #[schema(value_type =  chrono::DateTime<chrono::FixedOffset>, format = DateTime)]
    pub created_at: ConfigTimeStamp,
}
impl StorageConfigInner {
    pub fn test_config() -> Self {
        StorageConfigInner {
            storage_name: "test".into(),
            storage_id: Uuid::new_v4(),
            storage_type: "test".into(),
            created_at: ConfigTimeStamp::from(Utc::now()),
        }
    }
}
pub trait StorageTypeConfigTrait: Into<StorageTypeConfig> {
    fn from_type_config(dyn_config: StorageTypeConfig) -> Result<Self, InvalidConfigType>
    where
        Self: Sized;

    fn type_name(&self) -> &'static str;
}
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct StorageConfig {
    #[serde(flatten)]
    pub storage_config: StorageConfigInner,
    pub type_config: StorageTypeConfig,
}
impl TryFrom<DBStorage> for StorageConfig {
    type Error = StorageError;
    fn try_from(db_storage: DBStorage) -> Result<Self, Self::Error> {
        let type_config = serde_json::from_value(db_storage.config.0)?;
        Ok(StorageConfig {
            storage_config: StorageConfigInner {
                storage_name: db_storage.name.into(),
                storage_id: db_storage.id,
                storage_type: db_storage.storage_type,
                created_at: db_storage.created_at,
            },
            type_config,
        })
    }
}
impl From<BorrowedStorageConfig<'_>> for StorageConfig {
    fn from(borrowed: BorrowedStorageConfig) -> Self {
        StorageConfig {
            storage_config: borrowed.storage_config.clone(),
            type_config: borrowed.config.into(),
        }
    }
}
#[derive(Debug, Clone, Serialize)]
pub struct BorrowedStorageConfig<'a> {
    #[serde(flatten)]
    pub storage_config: &'a StorageConfigInner,
    pub config: BorrowedStorageTypeConfig<'a>,
}
macro_rules! storage_type_config {
    (
        $(
            $variant:ident($config:ty)
        ),*
    ) => {
        #[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
        #[serde(tag = "type", content = "settings")]
        pub enum StorageTypeConfig {
            $(
                $variant($config),
            )*
        }
        impl StorageTypeConfig {
            pub fn type_name(&self) -> &'static str {
                match self {
                    $(
                        StorageTypeConfig::$variant(_) => stringify!($variant),
                    )*
                }
            }
        }
        #[derive(Debug, Clone, Copy, Serialize)]
        #[serde(tag = "type", content = "settings")]
        pub enum BorrowedStorageTypeConfig<'a> {
            $(
                $variant(&'a $config),
            )*
        }
        impl From<BorrowedStorageTypeConfig<'_>> for StorageTypeConfig {
            fn from(borrowed: BorrowedStorageTypeConfig) -> Self {
                match borrowed {
                    $(
                        BorrowedStorageTypeConfig::$variant(local) => StorageTypeConfig::$variant(local.clone()),
                    )*
                }
            }
        }
        $(
            impl From<$config> for StorageTypeConfig {
                fn from(config: $config) -> Self {
                    StorageTypeConfig::$variant(config)
                }
            }
        )*
        $(
            impl StorageTypeConfigTrait for $config {
                fn from_type_config(dyn_config: StorageTypeConfig) -> Result<Self, InvalidConfigType>
                    where
                        Self: Sized{
                    match dyn_config {
                        StorageTypeConfig::$variant(config) => Ok(config),
                        _ => Err(InvalidConfigType(stringify!($config), dyn_config.type_name())),
                    }
                }
                fn type_name(&self) -> &'static str {
                    stringify!($config)
                }
            }
        )*
    };
}
storage_type_config! {
    Local(LocalConfig),
    S3(S3Config)
}

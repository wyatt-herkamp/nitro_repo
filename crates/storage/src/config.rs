use nr_core::{database::storage::DBStorage, ConfigTimeStamp};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{local::LocalConfig, StorageError};
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct StorageConfigInner {
    pub storage_name: String,
    pub storage_id: Uuid,
    pub storage_type: String,
    #[schema(value_type =  chrono::DateTime<chrono::FixedOffset>, format = DateTime)]
    pub created_at: ConfigTimeStamp,
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

    };
}
storage_type_config! {
    Local(LocalConfig)
}

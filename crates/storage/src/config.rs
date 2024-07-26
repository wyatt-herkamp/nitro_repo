use nr_core::ConfigTimeStamp;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::local::LocalConfig;
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfigInner {
    pub storage_name: String,
    pub storage_id: Uuid,
    pub created_at: ConfigTimeStamp,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    #[serde(flatten)]
    pub storage_config: StorageConfigInner,
    pub type_config: StorageTypeConfig,
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
        #[derive(Debug, Clone, Serialize, Deserialize)]
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

use async_trait::async_trait;
use bytes::Bytes;
use futures::Stream;
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::Value;
use tokio::sync::RwLockReadGuard;

use crate::repository::settings::{RepositoryConfig, RepositoryType};
use crate::storage::bad_storage::BadStorage;
use crate::storage::error::StorageError;
use crate::storage::file::{StorageFile, StorageFileResponse};
use crate::storage::local_storage::LocalStorage;
use crate::storage::models::{
    Storage, StorageConfig, StorageFactory, StorageSaver, StorageStatus, StorageType,
};

pub mod bad_storage;
pub mod dynamic;
pub mod error;
pub mod file;
pub mod local_storage;
pub mod models;
pub mod multi;
pub use dynamic::DynamicStorage;
pub static STORAGES_CONFIG: &str = "storages.nitro_repo";
pub static STORAGE_CONFIG: &str = "storage.nitro_repo";

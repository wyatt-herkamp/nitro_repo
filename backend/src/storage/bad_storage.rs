use async_trait::async_trait;
use bytes::Bytes;
use futures::Stream;
use log::warn;
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::Value;
use tokio::sync::RwLockReadGuard;

use crate::repository::settings::{RepositoryConfig, RepositoryType};
use crate::storage::error::StorageError;
use crate::storage::file::{StorageFile, StorageFileResponse};
use crate::storage::models::StorageStatus;
use crate::storage::StorageSaver;

/// This is a storage that is here to represent a storage that failed to load from the config stage
#[derive(Debug)]
pub struct BadStorage {
    pub factory: StorageSaver,
    pub status: StorageStatus,
}
impl BadStorage {
    pub fn create(factory: StorageSaver, error: StorageError) -> BadStorage {
        BadStorage {
            factory,
            status: StorageStatus::CreateError(error),
        }
    }
}

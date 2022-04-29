use crate::error::internal_error::InternalError;
use crate::repository::models::{Repository, RepositorySummary};
use crate::storage::models::{Storage, StorageFile, StorageFileResponse};
use actix_web::HttpRequest;
use either::Either;
use serde::{Deserialize, Serialize, Serializer};
use std::collections::HashMap;
use std::ptr::NonNull;
use crate::api_response::SiteResponse;
use crate::settings::models::StringMap;
use crate::storage::multi::{MultiStorageController, MultiStorageError};
use async_trait::async_trait;
use thiserror::Error;
use tokio::sync::RwLockReadGuard;
use crate::storage::StorageManager::MultiStorageHandler;
use crate::storage::StorageHandlerError::MissingImpl;

pub mod admin;
pub mod local_storage;
pub mod models;

#[cfg(feature = "multi_storage")]
pub mod multi;


pub static STORAGES_CONFIG: &str = "storages.nitro_repo";
pub static STORAGE_CONFIG: &str = "storage.nitro_repo";

#[derive(Error, Debug)]
pub enum StorageHandlerError {
    #[cfg(feature = "multi_storage")]
    #[error("{0}")]
    MultiStorageError(MultiStorageError),
    #[error("Mising Impl")]
    MissingImpl,
}

pub struct ValueRef<'a, T, O> {
    _guard: RwLockReadGuard<'a, O>,
    value: &'a T,
}

#[async_trait]
pub trait StorageHandlerType: Serialize {
    type Error;
    async fn get_storage_by_name(&self, name: &str) -> Result<Option<Storage>, Self::Error>;
    async fn create_storage(&self, storage: Storage) -> Result<(), Self::Error>;
    async fn delete_storage(&self, storage: &str) -> Result<bool, Self::Error>;
    async fn storages_as_file_list(&self) -> Result<Vec<StorageFile>, Self::Error>;
}

pub enum StorageManager {
    #[cfg(feature = "multi_storage")]
    MultiStorageHandler(MultiStorageController)
}

impl StorageManager {
    pub async fn init() -> Result<StorageManager, StorageHandlerError> {
        #[cfg(feature = "multi_storage")]
        {
            let storage_controller = MultiStorageController::init().await.map_err(StorageHandlerError::MultiStorageError)?;
            return Ok(MultiStorageHandler(storage_controller));
        }
        Err(MissingImpl)
    }
}

impl Serialize for StorageManager {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
    {
        return match self {
            MultiStorageHandler(handler) => {
                serializer.serialize_some(handler)
            }
        };
    }
}

#[async_trait]
impl StorageHandlerType for StorageManager {
    type Error = StorageHandlerError;


    async fn get_storage_by_name(&self, name: &str) -> Result<Option<Storage>, Self::Error> {
        match self {
            #[cfg(feature = "multi_storage")]
            MultiStorageHandler(handler) => {
                handler.get_storage_by_name(name).await.map_err(StorageHandlerError::MultiStorageError)
            }
        }
    }

    async fn create_storage(&self, storage: Storage) -> Result<(), Self::Error> {
        match self {
            #[cfg(feature = "multi_storage")]
            MultiStorageHandler(handler) => {
                handler.create_storage(storage).await.map_err(StorageHandlerError::MultiStorageError)
            }
        }
    }

    async fn delete_storage(&self, storage: &str) -> Result<bool, Self::Error> {
        match self {
            #[cfg(feature = "multi_storage")]
            MultiStorageHandler(handler) => {
                handler.delete_storage(storage).await.map_err(StorageHandlerError::MultiStorageError)
            }
        }
    }

    async fn storages_as_file_list(&self) -> Result<Vec<StorageFile>, Self::Error> {
        match self {
            #[cfg(feature = "multi_storage")]
            MultiStorageHandler(handler) => {
                handler.storages_as_file_list().await.map_err(StorageHandlerError::MultiStorageError)
            }
        }
    }
}

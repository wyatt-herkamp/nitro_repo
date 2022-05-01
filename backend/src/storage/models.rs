use actix_web::HttpRequest;
use either::{Either, Left, Right};
use std::collections::HashMap;
use std::fmt;
use std::fmt::{Debug};


use crate::api_response::SiteResponse;
use crate::storage::local_storage::{LocalStorage, LocalStorageError};
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};

use async_trait::async_trait;
use serde::de::{SeqAccess, Unexpected, Visitor};
use serde::ser::SerializeStruct;

use thiserror::Error;
use crate::error::internal_error::InternalError;
use crate::repository::data::{RepositoryDataType, RepositoryMainConfig, RepositorySetting, RepositoryType, RepositoryValue};
use crate::storage::handler::{StorageHandler, StorageHandlerFactory};


pub static STORAGE_FILE: &str = "storages.json";
pub static STORAGE_FILE_BAK: &str = "storages.json.bak";

#[derive(Error, Debug)]
pub enum StorageError {
    #[error("{0}")]
    LocalStorageError(LocalStorageError),
    #[error("{0}")]
    LoadFailure(String),
    #[error("IO error {0}")]
    IOError(std::io::Error),
    #[error("JSON error {0}")]
    JSONError(serde_json::Error),
    #[error("Storage Already Exists!")]
    StorageAlreadyExist,
}

impl From<std::io::Error> for StorageError {
    fn from(err: std::io::Error) -> StorageError {
        StorageError::IOError(err)
    }
}

impl From<LocalStorageError> for StorageError {
    fn from(err: LocalStorageError) -> StorageError {
        StorageError::LocalStorageError(err)
    }
}

impl From<serde_json::Error> for StorageError {
    fn from(err: serde_json::Error) -> StorageError {
        StorageError::JSONError(err)
    }
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    pub public_name: String,
    pub name: String,
    pub created: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnloadedStorage {
    pub storage_handler: StorageHandlerFactory,
    #[serde(flatten)]

    pub config: StorageConfig,
}

#[derive(Debug, Clone)]
pub struct Storage {
    pub storage_handler: StorageHandler,
    pub config: StorageConfig,
}

impl Storage {
    // Repository Handlers
    pub async fn create_repository(
        &self,
        name: String,
        repository_type: RepositoryType) -> Result<RepositoryValue, StorageError> {
        return match &self.storage_handler {
            StorageHandler::LocalStorage(local) => local
                .create_repository(&self.config, name, repository_type)
                .await
                .map_err(StorageError::LocalStorageError),
        };
    }
    pub async fn delete_repository<R: RepositoryDataType>(
        &self,
        repository: &R,
        delete_files: bool,
    ) -> Result<(), StorageError> {
        return match &self.storage_handler {
            StorageHandler::LocalStorage(local) => local
                .delete_repository(&self.config, repository, delete_files)
                .await
                .map_err(StorageError::LocalStorageError),
        };
    }
    pub async fn get_repositories(&self) -> Result<Vec<RepositoryValue>, StorageError> {
        return match &self.storage_handler {
            StorageHandler::LocalStorage(local) => local
                .get_repositories(&self.config)
                .await
                .map_err(StorageError::LocalStorageError),
        };
    }
    pub async fn get_repository<T: RepositorySetting>(&self, name: &str) -> Result<Option<RepositoryMainConfig<T>>, StorageError> {
        return match &self.storage_handler {
            StorageHandler::LocalStorage(local) => local
                .get_repository(&self.config, name)
                .await
                .map_err(StorageError::LocalStorageError),
        };
    }
    pub async fn update_repository<T: RepositorySetting>(&self, repository: RepositoryMainConfig<T>) -> Result<(), StorageError> {
        return match &self.storage_handler {
            StorageHandler::LocalStorage(local) => local
                .update_repository(&self.config, repository)
                .await
                .map_err(StorageError::LocalStorageError),
        };
    }
    //File Handlers
    pub async fn save_file<R: RepositoryDataType>(
        &self,
        repository: &R,
        file: &[u8],
        location: &str,
    ) -> Result<(), StorageError> {
        return match &self.storage_handler {
            StorageHandler::LocalStorage(local) => local
                .save_file(&self.config, repository, file, location)
                .await
                .map_err(StorageError::LocalStorageError),
        };
    }
    pub async fn delete_file<R: RepositoryDataType>(
        &self,
        repository: &R,
        location: &str,
    ) -> Result<(), StorageError> {
        return match &self.storage_handler {
            StorageHandler::LocalStorage(local) => local
                .delete_file(&self.config, repository, location)
                .await
                .map_err(StorageError::LocalStorageError),
        };
    }
    pub async fn get_file_as_response<R: RepositoryDataType>(
        &self,
        repository: &R,
        location: &str,
        request: &HttpRequest,
    ) -> Result<Option<FileResponse<SiteResponse>>, StorageError> {
        return match &self.storage_handler {
            StorageHandler::LocalStorage(local) => {
                let option = local
                    .get_file_as_response(&self.config, repository, location)
                    .await
                    .map_err(StorageError::LocalStorageError)?;
                if option.is_none() {
                    return Ok(None);
                }
                let value = option.unwrap();
                if value.is_left() {
                    Ok(Some(Left(value.left().unwrap().to_request(request))))
                } else {
                    Ok(Some(Right(value.right().unwrap())))
                }
            }
        };
    }
    pub async fn get_file<R: RepositoryDataType>(
        &self,
        repository: &R,
        location: &str,
    ) -> Result<Option<Vec<u8>>, StorageError> {
        return match &self.storage_handler {
            StorageHandler::LocalStorage(local) => local
                .get_file(&self.config, repository, location)
                .await
                .map_err(StorageError::LocalStorageError),
        };
    }
}

///Storage Files are just a data container holding the file name, directory relative to the root of nitro_repo and if its a directory
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct StorageFile {
    pub name: String,
    pub full_path: String,
    pub directory: bool,
    pub file_size: u64,
    pub created: u128,
}

/// StorageFileResponse is a trait that can be turned into a SiteResponse for example if its  LocalFile it will return into Actix's NamedFile response
pub trait StorageFileResponse {
    fn to_request(self, request: &HttpRequest) -> SiteResponse;
}

pub type FileResponse<T> = Either<T, Vec<StorageFile>>;

#[async_trait]
pub trait StorageType<T: StorageFileResponse> {
    type Error;
    type StorageConfig;
    // Initialize the Storage at Storage start.
    async fn load(&mut self, config: &StorageConfig) -> Result<(), Self::Error>;
    // Unload the storage
    fn unload(&mut self) -> Result<(), Self::Error>;

    // Repository Handlers
    async fn create_repository(
        &self,
        config: &StorageConfig,
        name: String, repository_type: RepositoryType,
    ) -> Result<RepositoryValue, Self::Error>;
    async fn delete_repository<R: RepositoryDataType>(
        &self,
        config: &StorageConfig,
        repository: &R,
        delete_files: bool,
    ) -> Result<(), Self::Error>;
    async fn get_repositories(
        &self,
        config: &StorageConfig,
    ) -> Result<Vec<RepositoryValue>, Self::Error>;
    async fn get_repository<RS: RepositorySetting>(
        &self,
        config: &StorageConfig,
        uuid: &str,
    ) -> Result<Option<RepositoryMainConfig<RS>>, Self::Error>;
    async fn update_repository<RS: RepositorySetting>(
        &self,
        config: &StorageConfig,
        repository: RepositoryMainConfig<RS>,
    ) -> Result<(), Self::Error>;
    //File Handlers
    async fn save_file<R: RepositoryDataType>(
        &self,
        config: &StorageConfig,
        repository: &R,
        file: &[u8],
        location: &str,
    ) -> Result<(), Self::Error>;
    async fn delete_file<R: RepositoryDataType>(
        &self,
        config: &StorageConfig,
        repository: &R,
        location: &str,
    ) -> Result<(), Self::Error>;
    async fn get_file_as_response<R: RepositoryDataType>(
        &self,
        config: &StorageConfig,
        repository: &R,
        location: &str,
    ) -> Result<Option<FileResponse<T>>, Self::Error>;
    async fn get_file<R: RepositoryDataType>(
        &self,
        config: &StorageConfig,
        repository: &R,
        location: &str,
    ) -> Result<Option<Vec<u8>>, Self::Error>;
}

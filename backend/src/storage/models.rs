use actix_web::HttpRequest;
use either::{Either, Left, Right};
use std::collections::HashMap;
use std::fmt::Debug;
use std::fs;
use std::fs::{read_to_string, OpenOptions};
use std::io::Write;
use std::path::Path;

use crate::error::internal_error::InternalError;
use crate::repository::models::{Repository, RepositorySummary};
use crate::storage::local_storage::{LocalStorage, LocalStorageError};
use serde::{Deserialize, Serialize};
use crate::api_response::SiteResponse;
use crate::settings::models::StringMap;
use async_trait::async_trait;
use sqlx::encode::IsNull::No;
use thiserror::Error;

pub static STORAGE_FILE: &str = "storages.json";
pub static STORAGE_FILE_BAK: &str = "storages.json.bak";

#[derive(Error, Debug)]
pub enum StorageError {
    #[error("{0}")]
    LocalStorageError(LocalStorageError),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StorageHandler {
    LocalStorage(LocalStorage),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    pub public_name: String,
    pub name: String,
    pub created: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Storage {
    pub storage_handler: StorageHandler,

    #[serde(flatten)]
    pub config: StorageConfig,
}

impl Storage {
    pub async fn init(&self) -> Result<(), StorageError> {
        return match &self.storage_handler {
            StorageHandler::LocalStorage(local) => {
                local.init(&self.config).await.map_err(StorageError::LocalStorageError)
            }
        };
    }
    // Repository Handlers
    pub async fn create_repository(
        &self,
        repository: RepositorySummary,
    ) -> Result<Repository, StorageError> {
        return match &self.storage_handler {
            StorageHandler::LocalStorage(local) => {
                local.create_repository(&self.config, repository).await.map_err(StorageError::LocalStorageError)
            }
        };
    }
    pub async fn delete_repository(
        &self,
        repository: &Repository,
        delete_files: bool,
    ) -> Result<(), StorageError> {
        return match &self.storage_handler {
            StorageHandler::LocalStorage(local) => {
                local.delete_repository(&self.config, repository, delete_files).await.map_err(StorageError::LocalStorageError)
            }
        };
    }
    pub async fn get_repositories(&self) -> Result<RepositoriesFile, StorageError> {
        return match &self.storage_handler {
            StorageHandler::LocalStorage(local) => {
                local.get_repositories(&self.config).await.map_err(StorageError::LocalStorageError)
            }
        };
    }
    pub async fn get_repository(
        &self,
        name: &str,
    ) -> Result<Option<Repository>, StorageError> {
        return match &self.storage_handler {
            StorageHandler::LocalStorage(local) => {
                local.get_repository(&self.config, name).await.map_err(StorageError::LocalStorageError)
            }
        };
    }
    pub async fn update_repository(
        &self,
        repository: &Repository,
    ) -> Result<(), StorageError> {
        return match &self.storage_handler {
            StorageHandler::LocalStorage(local) => {
                local.update_repository(&self.config, repository).await.map_err(StorageError::LocalStorageError)
            }
        };
    }
    //File Handlers
    pub async fn save_file(
        &self,
        repository: &Repository,
        file: &[u8],
        location: &str,
    ) -> Result<(), StorageError> {
        return match &self.storage_handler {
            StorageHandler::LocalStorage(local) => {
                local.save_file(&self.config, repository, file, location).await.map_err(StorageError::LocalStorageError)
            }
        };
    }
    pub async fn delete_file(
        &self,
        repository: &Repository,
        location: &str,
    ) -> Result<(), StorageError> {
        return match &self.storage_handler {
            StorageHandler::LocalStorage(local) => {
                local.delete_file(&self.config, repository, location).await.map_err(StorageError::LocalStorageError)
            }
        };
    }
    pub async fn get_file_as_response(
        &self,
        repository: &Repository,
        location: &str, request: &HttpRequest,
    ) -> Result<Option<FileResponse<SiteResponse>>, StorageError> {
        return match &self.storage_handler {
            StorageHandler::LocalStorage(local) => {
                let option = local.get_file_as_response(&self.config, repository, location).await.map_err(StorageError::LocalStorageError)?;
                if option.is_none() { return Ok(None); }
                let value = option.unwrap();
                if value.is_left() {
                    Ok(Some(Left(value.left().unwrap().to_request(request))))
                } else {
                    Ok(Some(Right(value.right().unwrap())))
                }
            }
        };
    }
    pub async fn get_file(
        &self,
        repository: &Repository,
        location: &str,
    ) -> Result<Option<Vec<u8>>, StorageError> {
        return match &self.storage_handler {
            StorageHandler::LocalStorage(local) => {
                local.get_file(&self.config, repository, location).await.map_err(StorageError::LocalStorageError)
            }
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


pub type RepositoriesFile = HashMap<String, RepositorySummary>;
pub type FileResponse<T> = Either<T, Vec<StorageFile>>;

#[async_trait]
pub trait StorageType<T: StorageFileResponse> {
    type Error;
    async fn init(&self, config: &StorageConfig) -> Result<(), Self::Error>;
    // Repository Handlers
    async fn create_repository(
        &self, config: &StorageConfig,
        repository: RepositorySummary,
    ) -> Result<Repository, Self::Error>;
    async fn delete_repository(
        &self, config: &StorageConfig,
        repository: &Repository,
        delete_files: bool,
    ) -> Result<(), Self::Error>;
    async fn get_repositories(&self, config: &StorageConfig,
    ) -> Result<RepositoriesFile, Self::Error>;
    async fn get_repository(
        &self, config: &StorageConfig,
        uuid: &str,
    ) -> Result<Option<Repository>, Self::Error>;
    async fn update_repository(
        &self, config: &StorageConfig,
        repository: &Repository,
    ) -> Result<(), Self::Error>;
    //File Handlers
    async fn save_file(
        &self, config: &StorageConfig,
        repository: &Repository,
        file: &[u8],
        location: &str,
    ) -> Result<(), Self::Error>;
    async fn delete_file(
        &self, config: &StorageConfig,
        repository: &Repository,
        location: &str,
    ) -> Result<(), Self::Error>;
    async fn get_file_as_response(
        &self, config: &StorageConfig,
        repository: &Repository,
        location: &str,
    ) -> Result<Option<FileResponse<T>>, Self::Error>;
    async fn get_file(
        &self, config: &StorageConfig,
        repository: &Repository,
        location: &str,
    ) -> Result<Option<Vec<u8>>, Self::Error>;
}

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
use crate::storage::local_storage::LocalStorage;
use crate::storage::{LocationHandler, RepositoriesFile, StorageFile, StorageFileResponse};
use crate::{SiteResponse, StringMap};
use serde::{Deserialize, Serialize};

pub static STORAGE_FILE: &str = "storages.json";
pub static STORAGE_FILE_BAK: &str = "storages.json.bak";

pub fn load_storages() -> anyhow::Result<Storages> {
    let path = Path::new(STORAGE_FILE);
    if !path.exists() {
        return Ok(HashMap::new());
    }
    let string = read_to_string(&path)?;
    let result: Storages = serde_json::from_str(&string)?;
    Ok(result)
}

pub fn save_storages(storages: &Storages) -> Result<(), InternalError> {
    let result = serde_json::to_string(&storages)?;
    let path = Path::new(STORAGE_FILE);
    let bak = Path::new(STORAGE_FILE_BAK);
    if bak.exists() {
        fs::remove_file(bak)?;
    }
    if path.exists() {
        fs::rename(path, bak)?;
    }
    let mut file = OpenOptions::new().write(true).create(true).open(path)?;
    file.write_all(result.as_bytes())?;
    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LocationType {
    LocalStorage,
}

pub type Storages = HashMap<String, Storage<StringMap>>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Storage<T> {
    pub public_name: String,
    pub name: String,
    pub created: i64,
    pub location_type: LocationType,
    #[serde(flatten)]
    pub location: T,
}

pub type StringStorage = Storage<StringMap>;

impl Storage<StringMap> {
    pub fn create_repository(
        &self,
        repository: RepositorySummary,
    ) -> Result<Repository, InternalError> {
        match self.location_type {
            LocationType::LocalStorage => LocalStorage::create_repository(self, repository),
        }
    }

    pub fn get_repositories(&self) -> Result<RepositoriesFile, InternalError> {
        match self.location_type {
            LocationType::LocalStorage => LocalStorage::get_repositories(self),
        }
    }

    pub fn get_repository(&self, name: &str) -> Result<Option<Repository>, InternalError> {
        match self.location_type {
            LocationType::LocalStorage => LocalStorage::get_repository(self, name),
        }
    }

    pub fn update_repository(&self, repository: &Repository) -> Result<(), InternalError> {
        match self.location_type {
            LocationType::LocalStorage => LocalStorage::update_repository(self, repository),
        }
    }

    pub fn save_file(
        &self,
        repository: &Repository,
        file: &[u8],
        location: &str,
    ) -> Result<(), InternalError> {
        match self.location_type {
            LocationType::LocalStorage => LocalStorage::save_file(self, repository, file, location),
        }
    }

    pub fn delete_file(
        &self,
        repository: &Repository,
        location: &str,
    ) -> Result<(), InternalError> {
        match self.location_type {
            LocationType::LocalStorage => LocalStorage::delete_file(self, repository, location),
        }
    }

    pub fn get_file_as_response(
        &self,
        repository: &Repository,
        location: &str,
        request: &HttpRequest,
    ) -> Result<Either<SiteResponse, Vec<StorageFile>>, InternalError> {
        match self.location_type {
            LocationType::LocalStorage => {
                let response = LocalStorage::get_file_as_response(self, repository, location)?;
                if response.is_left() {
                    Ok(Left(response.left().unwrap().to_request(request)))
                } else {
                    Ok(Right(response.right().unwrap()))
                }
            }
        }
    }

    pub fn get_file(
        &self,
        repository: &Repository,
        location: &str,
    ) -> Result<Option<Vec<u8>>, InternalError> {
        match self.location_type {
            LocationType::LocalStorage => LocalStorage::get_file(self, repository, location),
        }
    }
}

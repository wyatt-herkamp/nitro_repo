use std::collections::HashMap;
use std::fs::{create_dir_all, read_to_string, remove_file};
use std::path::{Path, PathBuf};
use crate::error::internal_error::InternalError;
use crate::StringMap;
use serde::{Serialize, Deserialize};
use crate::repository::models::{Repository, RepositorySummary};
use crate::storage::{LocationHandler, RepositoriesFile, STORAGE_CONFIG, StorageFile};
use crate::storage::models::{LocationType, Storage};



#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalStorage {
    pub location: String,
}

impl LocationHandler for LocalStorage {
    fn init(storage: &Storage<StringMap>) -> Result<(), InternalError> {
        let location = storage.location.get("location").unwrap();
        let path = Path::new(location);
        if !path.exists() {
            create_dir_all(&path)?;
        }
        let buf = path.join(STORAGE_CONFIG);
        if buf.exists() {
            remove_file(&buf)?
        }

        Ok(())
    }

    fn create_repository(storage: &Storage<StringMap>, repository: RepositorySummary) -> Result<Repository, InternalError> {
        todo!()
    }

    fn get_repositories(storage: &Storage<StringMap>) -> Result<RepositoriesFile, InternalError>
    {
        let location = storage.location.get("location").unwrap();

        let path = Path::new(location).join(STORAGE_CONFIG);
        if !path.exists() {
            return Ok(HashMap::new());
        }
        let string = read_to_string(&path)?;
        let result: RepositoriesFile = serde_json::from_str(&string)?;
        return Ok(result);
    }

    fn get_repository(storage: &Storage<StringMap>, repository: &str) -> Result<Option<Repository>, InternalError> {
        todo!()
    }

    fn update_repository(storage: &Storage<StringMap>, repository: &Repository) -> Result<(), InternalError> {
        todo!()
    }

    fn save_file(storage: &Storage<StringMap>, repository: &Repository, file: &[u8], location: String) -> Result<(), InternalError> {
        todo!()
    }

    fn delete_file(storage: &Storage<StringMap>, repository: &Repository, location: String) -> Result<(), InternalError> {
        todo!()
    }

    fn list_files(storage: &Storage<StringMap>, repository: &Repository, location: String) -> Result<Vec<StorageFile>, InternalError> {
        todo!()
    }

    fn get_file(storage: &Storage<StringMap>, repository: &Repository, location: String) -> Result<PathBuf, InternalError> {
        todo!()
    }
}

impl TryFrom<Storage<StringMap>> for Storage<LocalStorage> {
    type Error = InternalError;

    fn try_from(value: Storage<StringMap>) -> Result<Self, Self::Error> {
        Ok(Self {
            public_name: value.public_name,
            name: value.name,
            created: value.created,
            location_type: LocationType::LocalStorage,
            location: LocalStorage::try_from(value.location)?,
        })
    }
}

impl TryFrom<StringMap> for LocalStorage {
    type Error = InternalError;

    fn try_from(mut value: StringMap) -> Result<LocalStorage, Self::Error> {
        let location = value.remove("location").ok_or_else(|| InternalError::ConfigError("storage missing location".to_string()))?;

        return Ok(LocalStorage { location });
    }
}
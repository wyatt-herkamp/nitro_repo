use std::fs::{create_dir_all, remove_file};
use std::path::{Path};
use crate::error::internal_error::InternalError;
use crate::StringMap;
use serde::{Serialize, Deserialize};
use crate::repository::models::{Repository, RepositorySummary};
use crate::storage::{FileResponse, LocationHandler, STORAGE_CONFIG, StorageFile};
use crate::storage::models::Storage;

pub struct LocalFileResponse{
    pub path: std::path::PathBuf

}
impl FileResponse for LocalFileResponse{

}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalStorage {
    pub location: String,
}

impl LocationHandler<LocalFileResponse> for LocalStorage {
    fn init(storage: &Storage<LocalStorage>) -> Result<(), InternalError> {
        let path = Path::new(&storage.location.location);
        if !path.exists(){
            create_dir_all(&path)?;
        }
        let buf = path.join(STORAGE_CONFIG);
        if buf.exists(){
            remove_file(&buf)?
        }

        Ok(())
    }

    fn create_repository(storage: &Storage<LocalStorage>, repository: RepositorySummary) -> Result<Repository, InternalError> {
        todo!()
    }

    fn get_repositories(storage: &Storage<LocalStorage>, repository: &Repository) -> Result<Vec<String>, InternalError> {
        todo!()
    }

    fn get_repository(storage: &Storage<LocalStorage>, repository: &Repository) -> Result<Repository, InternalError> {
        todo!()
    }

    fn update_repository(storage: &Storage<LocalStorage>, repository: &Repository) -> Result<(), InternalError> {
        todo!()
    }

    fn save_file(storage: &Storage<LocalStorage>, repository: &Repository, file: &[u8], location: String) -> Result<(), InternalError> {
        todo!()
    }

    fn delete_file(storage: &Storage<LocalStorage>, repository: &Repository, location: String) -> Result<(), InternalError> {
        todo!()
    }

    fn list_files(storage: &Storage<LocalStorage>, repository: &Repository, location: String) -> Result<Vec<StorageFile>, InternalError> {
        todo!()
    }

    fn get_file(storage: &Storage<LocalStorage>, repository: &Repository, location: String) -> Result<LocalFileResponse, InternalError> {
        todo!()
    }
}

impl TryFrom<StringMap> for LocalStorage {
    type Error = InternalError;

    fn try_from(mut value: StringMap) -> Result<LocalStorage, Self::Error> {
        let location = value.remove("location").ok_or_else(|| InternalError::ConfigError("storage missing location".to_string()))?;

        return Ok(LocalStorage { location });
    }
}
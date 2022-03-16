use std::collections::HashMap;
use std::path::PathBuf;
use crate::error::internal_error::InternalError;
use crate::repository::models::{Repository, RepositorySummary};
use serde::{Serialize, Deserialize};
use crate::storage::models::Storage;
use crate::StringMap;

pub mod admin;
pub mod models;
pub mod local_storage;

pub static STORAGE_CONFIG: &str = "storages.nitro_repo";

pub type RepositoriesFile = HashMap<String, RepositorySummary>;

pub trait FileResponse {}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct StorageFile {
    pub name: String,
    pub full_path: String,
    pub directory: bool,
}

pub trait LocationHandler<Rhs = Self> {
    fn init(storage: &Storage<StringMap>) -> Result<(), InternalError>;
    // Repository Handlers
    fn create_repository(storage: &Storage<StringMap>, repository: RepositorySummary) -> Result<Repository, InternalError>;
    fn get_repositories(storage: &Storage<StringMap>) -> Result<RepositoriesFile, InternalError>;
    fn get_repository(storage: &Storage<StringMap>, uuid: &str) -> Result<Option<Repository>, InternalError>;
    fn update_repository(storage: &Storage<StringMap>, repository: &Repository) -> Result<(), InternalError>;
    //File Handlers
    fn save_file(storage: &Storage<StringMap>, repository: &Repository, file: &[u8], location: String) -> Result<(), InternalError>;
    fn delete_file(storage: &Storage<StringMap>, repository: &Repository, location: String) -> Result<(), InternalError>;
    fn list_files(storage: &Storage<StringMap>, repository: &Repository, location: String) -> Result<Vec<StorageFile>, InternalError>;
    fn get_file(storage: &Storage<StringMap>, repository: &Repository, location: String) -> Result<PathBuf, InternalError>;
}


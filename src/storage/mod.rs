use crate::error::internal_error::InternalError;
use crate::repository::models::{Repository, RepositorySummary};
use serde::{Serialize, Deserialize};
use crate::storage::models::Storage;

pub mod admin;
pub mod models;
pub mod local_storage;
pub static STORAGE_CONFIG: &str = "storages.nitro_repo";

pub trait FileResponse {
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct StorageFile {
    pub name: String,
    pub full_path: String,
    pub directory: bool,
}

pub trait LocationHandler<T: FileResponse,Rhs=Self> {

    fn init(storage: &Storage<Rhs>) -> Result<(), InternalError>;
    // Repository Handlers
    fn create_repository(storage: &Storage<Rhs>, repository: RepositorySummary) -> Result<Repository, InternalError>;
    fn get_repositories(storage: &Storage<Rhs>, repository: &Repository) -> Result<Vec<String>, InternalError>;
    fn get_repository(storage: &Storage<Rhs>, repository: &Repository) -> Result<Repository, InternalError>;
    fn update_repository(storage: &Storage<Rhs>, repository: &Repository) -> Result<(), InternalError>;
    //File Handlers
    fn save_file(storage: &Storage<Rhs>, repository: &Repository, file: &[u8], location: String) -> Result<(), InternalError>;
    fn delete_file(storage: &Storage<Rhs>, repository: &Repository, location: String) -> Result<(), InternalError>;
    fn list_files(storage: &Storage<Rhs>, repository: &Repository, location: String) -> Result<Vec<StorageFile>, InternalError>;
    fn get_file(storage: &Storage<Rhs>, repository: &Repository, location: String) -> Result<T, InternalError>;
}


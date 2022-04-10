use crate::error::internal_error::InternalError;
use crate::repository::models::{Repository, RepositorySummary};
use crate::storage::models::Storage;
use crate::{SiteResponse, StringMap};
use actix_web::HttpRequest;
use either::Either;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub mod admin;
pub mod local_storage;
pub mod models;

pub static STORAGE_CONFIG: &str = "storages.nitro_repo";

pub type RepositoriesFile = HashMap<String, RepositorySummary>;
pub type FileResponse<T> = Either<T, Vec<StorageFile>>;

///Storage Files are just a data container holding the file name, directory relative to the root of nitro_repo and if its a directory
///
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

pub trait LocationHandler<T: StorageFileResponse, Rhs = Self> {
    fn init(storage: &Storage<StringMap>) -> Result<(), InternalError>;
    // Repository Handlers
    fn create_repository(
        storage: &Storage<StringMap>,
        repository: RepositorySummary,
    ) -> Result<Repository, InternalError>;
    fn delete_repository(
        storage: &Storage<StringMap>,
        repository: &Repository,
        delete_files: bool,
    ) -> Result<(), InternalError>;
    fn get_repositories(storage: &Storage<StringMap>) -> Result<RepositoriesFile, InternalError>;
    fn get_repository(
        storage: &Storage<StringMap>,
        uuid: &str,
    ) -> Result<Option<Repository>, InternalError>;
    fn update_repository(
        storage: &Storage<StringMap>,
        repository: &Repository,
    ) -> Result<(), InternalError>;
    //File Handlers
    fn save_file(
        storage: &Storage<StringMap>,
        repository: &Repository,
        file: &[u8],
        location: &str,
    ) -> Result<(), InternalError>;
    fn delete_file(
        storage: &Storage<StringMap>,
        repository: &Repository,
        location: &str,
    ) -> Result<(), InternalError>;
    fn get_file_as_response(
        storage: &Storage<StringMap>,
        repository: &Repository,
        location: &str,
    ) -> Result<FileResponse<T>, InternalError>;
    fn get_file(
        storage: &Storage<StringMap>,
        repository: &Repository,
        location: &str,
    ) -> Result<Option<Vec<u8>>, InternalError>;
}

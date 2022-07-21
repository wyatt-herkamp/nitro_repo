use actix_web::web::Bytes;
use actix_web::{web, HttpRequest};
use sea_orm::DatabaseConnection;
use serde::Deserialize;

use crate::authentication::Authentication;

use crate::repository::handler::Repository;
use crate::repository::response::RepoResponse;
use crate::storage::models::Storage;
use crate::storage::multi::MultiStorageController;
use crate::storage::DynamicStorage;
#[derive(Deserialize, Clone)]
pub struct GetPath {
    pub storage: String,
    pub repository: String,
    #[serde(default)]
    pub file: String,
}
impl GetPath {
    pub fn into_inner(self) -> (String, String, String) {
        (self.storage, self.repository, self.file)
    }
}
pub async fn get_repository(
    pool: web::Data<DatabaseConnection>,
    storages: web::Data<MultiStorageController<DynamicStorage>>,
    auth: Authentication,
    r: HttpRequest,
    path: web::Path<GetPath>,
) -> actix_web::Result<RepoResponse> {
    let (storage_name, repository_name, file) = path.into_inner().into_inner();
    let storage = crate::helpers::get_storage!(storages, storage_name);
    let repository = crate::helpers::get_repository!(storage, repository_name);
    repository
        .handle_get(&file, r.headers(), pool.get_ref(), auth)
        .await
}

pub async fn put_repository(
    pool: web::Data<DatabaseConnection>,
    storages: web::Data<MultiStorageController<DynamicStorage>>,
    auth: Authentication,
    r: HttpRequest,
    path: web::Path<GetPath>,
    bytes: Bytes,
) -> actix_web::Result<RepoResponse> {
    let (storage_name, repository_name, file) = path.into_inner().into_inner();
    let storage = crate::helpers::get_storage!(storages, storage_name);
    let repository = crate::helpers::get_repository!(storage, repository_name);
    repository
        .handle_put(&file, r.headers(), pool.get_ref(), auth, bytes)
        .await
}

pub async fn head_repository(
    pool: web::Data<DatabaseConnection>,
    storages: web::Data<MultiStorageController<DynamicStorage>>,
    auth: Authentication,
    r: HttpRequest,
    path: web::Path<GetPath>,
) -> actix_web::Result<RepoResponse> {
    let (storage_name, repository_name, file) = path.into_inner().into_inner();
    let storage = crate::helpers::get_storage!(storages, storage_name);
    let repository = crate::helpers::get_repository!(storage, repository_name);
    repository
        .handle_head(&file, r.headers(), pool.get_ref(), auth)
        .await
}

pub async fn post_repository(
    pool: web::Data<DatabaseConnection>,
    storages: web::Data<MultiStorageController<DynamicStorage>>,
    auth: Authentication,
    r: HttpRequest,
    path: web::Path<GetPath>,
    bytes: Bytes,
) -> actix_web::Result<RepoResponse> {
    let (storage_name, repository_name, file) = path.into_inner().into_inner();
    let storage = crate::helpers::get_storage!(storages, storage_name);
    let repository = crate::helpers::get_repository!(storage, repository_name);
    repository
        .handle_put(&file, r.headers(), pool.get_ref(), auth, bytes)
        .await
}

pub async fn patch_repository(
    pool: web::Data<DatabaseConnection>,
    storages: web::Data<MultiStorageController<DynamicStorage>>,
    auth: Authentication,
    r: HttpRequest,
    path: web::Path<GetPath>,
    bytes: Bytes,
) -> actix_web::Result<RepoResponse> {
    let (storage_name, repository_name, file) = path.into_inner().into_inner();
    let storage = crate::helpers::get_storage!(storages, storage_name);
    let repository = crate::helpers::get_repository!(storage, repository_name);
    repository
        .handle_patch(&file, r.headers(), pool.get_ref(), auth, bytes)
        .await
}

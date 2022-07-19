use actix_web::web::Bytes;
use actix_web::{web, HttpRequest};
use sea_orm::DatabaseConnection;
use serde::Deserialize;
use std::ops::Deref;

use crate::authentication::Authentication;
use crate::error::api_error::APIError;

use crate::repository::get_repository_handler;
use crate::repository::handler::RepositoryHandler;
use crate::repository::response::RepoResponse;
use crate::storage::models::Storage;
use crate::storage::multi::MultiStorageController;
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
    storages: web::Data<MultiStorageController>,
    auth: Authentication,
    r: HttpRequest,
    path: web::Path<GetPath>,
) -> actix_web::Result<RepoResponse> {
    let (storage_name, repository_name, file) = path.into_inner().into_inner();
    let storage = crate::helpers::get_storage!(storages, storage_name);
    let repository = crate::helpers::get_repository!(storage, repository_name)
        .deref()
        .clone();
    let repository_handler = get_repository_handler(storage, repository)
        .await?
        .ok_or_else(APIError::repository_not_found)?;
    repository_handler
        .handle_get(&file, r.headers(), pool.get_ref(), auth)
        .await
}

pub async fn put_repository(
    pool: web::Data<DatabaseConnection>,
    storages: web::Data<MultiStorageController>,
    auth: Authentication,
    r: HttpRequest,
    path: web::Path<GetPath>,
    bytes: Bytes,
) -> actix_web::Result<RepoResponse> {
    let (storage_name, repository_name, file) = path.into_inner().into_inner();
    let storage = crate::helpers::get_storage!(storages, storage_name);
    let repository = crate::helpers::get_repository!(storage, repository_name)
        .deref()
        .clone();
    let repository_handler = get_repository_handler(storage, repository)
        .await?
        .ok_or_else(APIError::repository_not_found)?;
    repository_handler
        .handle_put(&file, r.headers(), pool.get_ref(), auth, bytes)
        .await
}

pub async fn head_repository(
    pool: web::Data<DatabaseConnection>,
    storages: web::Data<MultiStorageController>,
    auth: Authentication,
    r: HttpRequest,
    path: web::Path<GetPath>,
) -> actix_web::Result<RepoResponse> {
    let (storage_name, repository_name, file) = path.into_inner().into_inner();
    let storage = crate::helpers::get_storage!(storages, storage_name);
    let repository = crate::helpers::get_repository!(storage, repository_name)
        .deref()
        .clone();
    let repository_handler = get_repository_handler(storage, repository)
        .await?
        .ok_or_else(APIError::repository_not_found)?;
    repository_handler
        .handle_head(&file, r.headers(), pool.get_ref(), auth)
        .await
}

pub async fn post_repository(
    pool: web::Data<DatabaseConnection>,
    storages: web::Data<MultiStorageController>,
    auth: Authentication,
    r: HttpRequest,
    path: web::Path<GetPath>,
    bytes: Bytes,
) -> actix_web::Result<RepoResponse> {
    let (storage_name, repository_name, file) = path.into_inner().into_inner();
    let storage = crate::helpers::get_storage!(storages, storage_name);
    let repository = crate::helpers::get_repository!(storage, repository_name)
        .deref()
        .clone();
    let repository_handler = get_repository_handler(storage, repository)
        .await?
        .ok_or_else(APIError::repository_not_found)?;
    repository_handler
        .handle_post(&file, r.headers(), pool.get_ref(), auth, bytes)
        .await
}

pub async fn patch_repository(
    pool: web::Data<DatabaseConnection>,
    storages: web::Data<MultiStorageController>,
    auth: Authentication,
    r: HttpRequest,
    path: web::Path<GetPath>,
    bytes: Bytes,
) -> actix_web::Result<RepoResponse> {
    let (storage_name, repository_name, file) = path.into_inner().into_inner();
    let storage = crate::helpers::get_storage!(storages, storage_name);
    let repository = crate::helpers::get_repository!(storage, repository_name)
        .deref()
        .clone();
    let repository_handler = get_repository_handler(storage, repository)
        .await?
        .ok_or_else(APIError::repository_not_found)?;
    repository_handler
        .handle_patch(&file, r.headers(), pool.get_ref(), auth, bytes)
        .await
}

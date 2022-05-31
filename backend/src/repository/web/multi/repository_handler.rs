use actix_web::web::Bytes;
use actix_web::{web, HttpRequest};
use sea_orm::DatabaseConnection;
use serde::Deserialize;

use crate::authentication::Authentication;
use crate::error::api_error::APIError;
use crate::error::internal_error::InternalError;
use crate::repository::get_repository_handler;
use crate::repository::handler::RepositoryHandler;
use crate::repository::response::RepoResponse;
use crate::storage::multi::MultiStorageController;

#[derive(Deserialize, Clone)]
pub struct GetPath {
    pub storage: String,
    pub repository: String,
    #[serde(default)]
    pub file: String,
}

pub async fn get_repository(
    pool: web::Data<DatabaseConnection>,
    storages: web::Data<MultiStorageController>,
    auth: Authentication,
    r: HttpRequest,
    path: web::Path<GetPath>,
) -> actix_web::Result<RepoResponse> {
    let storage = storages
        .get_storage_by_name(&path.storage)
        .await
        .map_err(InternalError::from)?
        .ok_or_else(APIError::storage_not_found)?;
    let repository_handler = get_repository_handler(storage, path.repository.as_str())
        .await?
        .ok_or_else(APIError::repository_not_found)?;
    repository_handler
        .handle_get(path.file.as_str(), r.headers(), pool.get_ref(), auth)
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
    let storage = storages
        .get_storage_by_name(&path.storage)
        .await
        .map_err(InternalError::from)?
        .ok_or_else(APIError::storage_not_found)?;
    let repository_handler = get_repository_handler(storage, path.repository.as_str())
        .await?
        .ok_or_else(APIError::repository_not_found)?;
    repository_handler
        .handle_put(path.file.as_str(), r.headers(), pool.get_ref(), auth, bytes)
        .await
}

pub async fn head_repository(
    pool: web::Data<DatabaseConnection>,
    storages: web::Data<MultiStorageController>,
    auth: Authentication,
    r: HttpRequest,
    path: web::Path<GetPath>,
) -> actix_web::Result<RepoResponse> {
    let storage = storages
        .get_storage_by_name(&path.storage)
        .await
        .map_err(InternalError::from)?
        .ok_or_else(APIError::storage_not_found)?;
    let repository_handler = get_repository_handler(storage, path.repository.as_str())
        .await?
        .ok_or_else(APIError::repository_not_found)?;
    repository_handler
        .handle_head(path.file.as_str(), r.headers(), pool.get_ref(), auth)
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
    let storage = storages
        .get_storage_by_name(&path.storage)
        .await
        .map_err(InternalError::from)?
        .ok_or_else(APIError::storage_not_found)?;
    let repository_handler = get_repository_handler(storage, path.repository.as_str())
        .await?
        .ok_or_else(APIError::repository_not_found)?;
    repository_handler
        .handle_post(path.file.as_str(), r.headers(), pool.get_ref(), auth, bytes)
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
    let storage = storages
        .get_storage_by_name(&path.storage)
        .await
        .map_err(InternalError::from)?
        .ok_or_else(APIError::storage_not_found)?;
    let repository_handler = get_repository_handler(storage, path.repository.as_str())
        .await?
        .ok_or_else(APIError::repository_not_found)?;
    repository_handler
        .handle_patch(path.file.as_str(), r.headers(), pool.get_ref(), auth, bytes)
        .await
}

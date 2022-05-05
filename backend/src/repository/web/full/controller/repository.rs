use crate::api_response::APIError;
use actix_web::web::Bytes;
use actix_web::{get, web, HttpRequest, ResponseError};
use log::{debug, trace};
use sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};

use crate::authentication::Authentication;

use crate::repository::nitro::{NitroFileResponse, NitroFileResponseType};
use crate::repository::response::RepoResponse;

use crate::storage::multi::MultiStorageController;

use crate::repository::handler::RepositoryHandler;
use crate::repository::web::full::to_request;
use crate::NitroRepoData;

pub async fn get_repository(
    database: web::Data<DatabaseConnection>,
    site: NitroRepoData,
    storages: web::Data<MultiStorageController>,
    r: HttpRequest,
    auth: Authentication,
    path: web::Path<(String, String, String)>,
) -> Result<RepoResponse, APIError> {
    let (storage, repository, file) = path.into_inner();
    debug!("GET in {}/{}: Route: {}", &storage, &repository, &file);
    return match to_request(&storage, &repository, &storages).await {
        Ok(ok) => {
            ok.handle_get(&file, r.headers(), database.as_ref(), auth)
                .await
        }
        Err(error) => Err(error.into()),
    };
}

pub async fn post_repository(
    database: web::Data<DatabaseConnection>,
    site: NitroRepoData,
    storages: web::Data<MultiStorageController>,
    r: HttpRequest,
    path: web::Path<(String, String, String)>,
    bytes: Bytes,
    auth: Authentication,
) -> Result<RepoResponse, APIError> {
    let (storage, repository, file) = path.into_inner();
    debug!("POST in {}/{}: Route: {}", &storage, &repository, &file);

    return match to_request(&storage, &repository, &storages).await {
        Ok(ok) => {
            ok.handle_post(&file, r.headers(), database.as_ref(), auth, bytes)
                .await
        }
        Err(error) => Err(error.into()),
    };
}

pub async fn patch_repository(
    database: web::Data<DatabaseConnection>,
    site: NitroRepoData,
    storages: web::Data<MultiStorageController>,
    r: HttpRequest,
    path: web::Path<(String, String, String)>,
    bytes: Bytes,
    auth: Authentication,
) -> Result<RepoResponse, APIError> {
    let (storage, repository, file) = path.into_inner();
    debug!("PATCH in {}/{}: Route: {}", &storage, &repository, &file);

    return match to_request(&storage, &repository, &storages).await {
        Ok(ok) => {
            ok.handle_patch(&file, r.headers(), database.as_ref(), auth, bytes)
                .await
        }
        Err(error) => Err(error.into()),
    };
}

pub async fn put_repository(
    database: web::Data<DatabaseConnection>,
    site: NitroRepoData,
    storages: web::Data<MultiStorageController>,
    r: HttpRequest,
    path: web::Path<(String, String, String)>,
    bytes: Bytes,
    auth: Authentication,
) -> Result<RepoResponse, APIError> {
    let (storage, repository, file) = path.into_inner();
    debug!("PUT in {}/{}: Route: {}", &storage, &repository, &file);

    return match to_request(&storage, &repository, &storages).await {
        Ok(ok) => {
            ok.handle_put(&file, r.headers(), database.as_ref(), auth, bytes)
                .await
        }
        Err(error) => Err(error.into()),
    };
}

pub async fn head_repository(
    database: web::Data<DatabaseConnection>,
    site: NitroRepoData,
    storages: web::Data<MultiStorageController>,
    r: HttpRequest,
    path: web::Path<(String, String, String)>,
    auth: Authentication,
) -> Result<RepoResponse, APIError> {
    let (storage, repository, file) = path.into_inner();
    debug!("HEAD in {}/{}: Route: {}", &storage, &repository, &file);

    return match to_request(&storage, &repository, &storages).await {
        Ok(ok) => {
            ok.handle_head(&file, r.headers(), database.as_ref(), auth)
                .await
        }
        Err(error) => Err(error.into()),
    };
}

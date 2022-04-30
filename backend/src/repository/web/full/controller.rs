use actix_web::http::StatusCode;
use actix_web::web::Bytes;
use actix_web::{get, web, HttpRequest, HttpResponse};

use log::{debug, trace};
use sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};

use crate::api_response::{APIResponse, SiteResponse};
use crate::error::internal_error::InternalError;
use crate::error::response::{bad_request, i_am_a_teapot, not_found};
use crate::repository::models::Repository;

use crate::authentication::Authentication;
use crate::repository::nitro::{NitroFile, NitroFileResponse, ResponseType};
use crate::repository::types::{RepoResponse, RepositoryRequest};
use crate::storage::models::StorageFile;
use crate::utils::get_accept;
use crate::NitroRepoData;
use crate::repository::web::full::{handle_result, to_request};
use crate::storage::multi::MultiStorageController;


pub async fn generate_storage_list(
    _site: NitroRepoData,
    storages: &MultiStorageController,
) -> Result<NitroFileResponse, InternalError> {
    let files = storages
        .storages_as_file_list()
        .await?
        .iter()
        .map(|value| NitroFile {
            response_type: ResponseType::Storage,
            file: value.to_owned(),
        })
        .collect();
    Ok(NitroFileResponse {
        files,
        response_type: ResponseType::Other,
        active_dir: "".to_string(),
    })
}

pub async fn browse(
    site: NitroRepoData,
    storages: web::Data<MultiStorageController>,
    r: HttpRequest,
) -> SiteResponse {
    APIResponse::respond_new(Some(generate_storage_list(site, &storages).await?), &r)
}

#[get("/storages/{storage}")]
pub async fn browse_storage(
    site: NitroRepoData,
    r: HttpRequest,
    path: web::Path<String>,
    storages: web::Data<MultiStorageController>,
) -> SiteResponse {
    let string = path.into_inner();
    println!("HEY");
    if string.is_empty() {
        return APIResponse::respond_new(Some(generate_storage_list(site, &storages).await?), &r);
    }

    let storage = storages.get_storage_by_name(&string).await?;
    if storage.is_none() {
        trace!("Storage {} not found", &string);
        return Err(InternalError::NotFound);
    }
    let storage = storage.unwrap();
    let map = storage.get_repositories().await?;
    let mut repos = NitroFileResponse {
        files: vec![],
        response_type: ResponseType::Storage,
        active_dir: string.clone(),
    };
    for (name, sum) in map {
        repos.files.push(NitroFile {
            response_type: ResponseType::Repository(sum),
            file: StorageFile {
                name: name.clone(),
                full_path: format!("{}/{}", &storage.config.name, &name),
                directory: true,
                file_size: 0,
                created: 0,
            },
        });
    }
    APIResponse::respond_new(Some(repos), &r)
}

#[derive(Serialize, Deserialize, Clone)]
pub struct GetPath {
    pub storage: String,
    pub repository: String,
    pub file: Option<String>,
}

pub async fn get_repository(
    connection: web::Data<DatabaseConnection>,
    site: NitroRepoData,
    r: HttpRequest,
    path: web::Path<GetPath>,
    auth: Authentication,
    storages: web::Data<MultiStorageController>,
) -> SiteResponse {
    let path = path.into_inner();
    let file = path.file.unwrap_or_else(|| "".to_string());
    let result = to_request(path.storage, path.repository, file,  storages).await;
    if let Err(error) = result {
        return match error {
            InternalError::NotFound => not_found(),
            _ => Err(error),
        };
    }
    let request = result.unwrap();
    let x = request
        .repository
        .repo_type
        .handle_get(&request, &r, connection.as_ref(), auth)
        .await?;

    handle_result(x, r)
}


pub async fn post_repository(
    connection: web::Data<DatabaseConnection>,
    site: NitroRepoData,
    r: HttpRequest,
    path: web::Path<(String, String, String)>,
    storages: web::Data<MultiStorageController>,
    bytes: Bytes,
) -> SiteResponse {
    let (storage, repository, file) = path.into_inner();
    let result = to_request(storage, repository, file, storages).await;
    if let Err(error) = result {
        return match error {
            InternalError::NotFound => not_found(),
            _ => Err(error),
        };
    }
    let request = result.unwrap();
    debug!(
        "POST {} in {}/{}: Route: {}",
        &request.repository.repo_type.to_string(),
        &request.storage.config.name,
        &request.repository.name,
        &request.value
    );
    let x = request
        .repository
        .repo_type
        .handle_post(&request, &r, connection.get_ref(), bytes)
        .await?;

    handle_result(x, r)
}

pub async fn patch_repository(
    connection: web::Data<DatabaseConnection>,
    site: NitroRepoData,
    r: HttpRequest,
    path: web::Path<(String, String, String)>,
    bytes: Bytes,
    storages: web::Data<MultiStorageController>,
) -> SiteResponse {
    let (storage, repository, file) = path.into_inner();
    let result = to_request(storage, repository, file, storages).await;
    if let Err(error) = result {
        return match error {
            InternalError::NotFound => not_found(),
            _ => Err(error),
        };
    }
    let request = result.unwrap();
    debug!(
        "PATCH {} in {}/{}: Route: {}",
        &request.repository.repo_type.to_string(),
        &request.storage.config.name,
        &request.repository.name,
        &request.value
    );
    let x = request
        .repository
        .repo_type
        .handle_patch(&request, &r, connection.get_ref(), bytes)
        .await?;

    handle_result(x, r)
}

pub async fn put_repository(
    connection: web::Data<DatabaseConnection>,
    site: NitroRepoData,
    r: HttpRequest,
    path: web::Path<(String, String, String)>,
    bytes: Bytes,
    auth: Authentication,
    storages: web::Data<MultiStorageController>,
) -> SiteResponse {
    let (storage, repository, file) = path.into_inner();
    let result = to_request(storage, repository, file, storages).await;
    if let Err(error) = result {
        return match error {
            InternalError::NotFound => not_found(),
            _ => Err(error),
        };
    }
    let request = result.unwrap();
    debug!(
        "PUT {} in {}/{}: Route: {}",
        &request.repository.repo_type.to_string(),
        &request.storage.config.name,
        &request.repository.name,
        &request.value
    );
    let x = request
        .repository
        .repo_type
        .handle_put(&request, &r, connection.get_ref(), bytes, auth)
        .await?;

    handle_result(x, r)
}

pub async fn head_repository(
    connection: web::Data<DatabaseConnection>,
    site: NitroRepoData,
    r: HttpRequest,
    path: web::Path<(String, String, String)>,
    storages: web::Data<MultiStorageController>,
) -> SiteResponse {
    let (storage, repository, file) = path.into_inner();
    let result = to_request(storage, repository, file, storages).await;
    if let Err(error) = result {
        return match error {
            InternalError::NotFound => not_found(),
            _ => Err(error),
        };
    }
    let request = result.unwrap();
    debug!(
        "HEAD {} in {}/{}: Route: {}",
        &request.repository.repo_type.to_string(),
        &request.storage.config.name,
        &request.repository.name,
        &request.value
    );
    let x = request
        .repository
        .repo_type
        .handle_head(&request, &r, &connection)
        .await?;

    handle_result(x, r)
}

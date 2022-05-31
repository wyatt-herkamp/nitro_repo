use std::ops::Deref;

use actix_web::http::StatusCode;
use actix_web::{delete, get, post, web, HttpResponse};
use sea_orm::DatabaseConnection;
use serde::Deserialize;

use crate::authentication::Authentication;
use crate::error::api_error::APIError;
use crate::error::internal_error::InternalError;
use crate::repository::web::RepositoryResponse;
use crate::repository::RepositoryType;
use crate::storage::error::StorageError;
use crate::storage::models::Storage;
use crate::storage::multi::MultiStorageController;
use crate::system::permissions::options::CanIDo;
use crate::system::user::UserModel;

/// Get all repositories from the storage
#[get("/repositories/{storage_name}")]
pub async fn get_repositories(
    storage_handler: web::Data<MultiStorageController>,
    database: web::Data<DatabaseConnection>,
    auth: Authentication,
    storage_name: web::Path<String>,
) -> actix_web::Result<HttpResponse> {
    let user: UserModel = auth.get_user(&database).await??;
    user.can_i_edit_repos()?;

    let storage = storage_handler
        .get_storage_by_name(&storage_name.into_inner())
        .await
        .map_err(InternalError::from)?
        .ok_or_else(|| APIError::from(("Storage not found", StatusCode::NOT_FOUND)))?;

    Ok(HttpResponse::Ok().json(
        storage
            .get_repositories()
            .await
            .map_err(InternalError::from)?,
    ))
}

/// Create a new repository
#[post("/repositories/{storage_name}/new/{repository_name}/{repository_type}")]
pub async fn create_repository(
    storage_handler: web::Data<MultiStorageController>,
    database: web::Data<DatabaseConnection>,
    auth: Authentication,
    storage_name: web::Path<String>,
    query_params: web::Path<(String, RepositoryType)>,
) -> actix_web::Result<HttpResponse> {
    let user: UserModel = auth.get_user(&database).await??;
    user.can_i_edit_repos()?;

    let storage = storage_handler
        .get_storage_by_name(&storage_name.into_inner())
        .await
        .map_err(InternalError::from)?
        .ok_or_else(|| APIError::from(("Storage not found", StatusCode::NOT_FOUND)))?;

    let (repository_name, repository_type) = query_params.into_inner();

    if let Err(error) = storage
        .create_repository(repository_name, repository_type)
        .await
    {
        return match error {
            StorageError::RepositoryAlreadyExists => {
                Err(APIError::from(("Repository already exists", StatusCode::CONFLICT)).into())
            }
            value => Err(InternalError::from(value).into()),
        };
    }

    Ok(HttpResponse::NoContent().finish())
}

#[derive(Deserialize)]
pub struct GetRepositoryQuery {
    #[serde(default)]
    pub all_info: bool,
}

/// Get a repository by the name and storage name
/// If the query param all_info is present. It will include other repository configs such as Frontend and Badge
#[get("/repositories/{storage_name}/{repository_name}")]
pub async fn get_repository(
    storage_handler: web::Data<MultiStorageController>,
    database: web::Data<DatabaseConnection>,
    auth: Authentication,
    path_params: web::Path<(String, String)>,
    query_params: web::Query<GetRepositoryQuery>,
) -> actix_web::Result<HttpResponse> {
    let user: UserModel = auth.get_user(&database).await??;
    user.can_i_edit_repos()?;
    let (storage_name, repository_name) = path_params.into_inner();
    let storage = storage_handler
        .get_storage_by_name(&storage_name)
        .await
        .map_err(InternalError::from)?
        .ok_or_else(|| APIError::from(("Storage not found", StatusCode::NOT_FOUND)))?;
    let repository = storage
        .get_repository(&repository_name)
        .await
        .map_err(InternalError::from)?
        .ok_or_else(|| APIError::from(("Repository not found", StatusCode::NOT_FOUND)))?;
    // Check if the query param contains all_info
    if query_params.all_info {
        //Generate a RepositoryResponse
        let response = RepositoryResponse::new(&repository, storage.deref()).await?;
        Ok(HttpResponse::Ok().json(response))
    } else {
        Ok(HttpResponse::Ok().json(repository.deref()))
    }
}

#[derive(Deserialize)]
pub struct DeleteRepositoryQuery {
    #[serde(default)]
    pub purge_repository: bool,
}

#[delete("/repositories/{storage_name}/{repository_name}")]
pub async fn delete_repository(
    storage_handler: web::Data<MultiStorageController>,
    database: web::Data<DatabaseConnection>,
    auth: Authentication,
    path_params: web::Path<(String, String)>,
    query_params: web::Query<DeleteRepositoryQuery>,
) -> actix_web::Result<HttpResponse> {
    let user: UserModel = auth.get_user(&database).await??;
    user.can_i_edit_repos()?;
    let (storage_name, repository_name) = path_params.into_inner();
    let storage = storage_handler
        .get_storage_by_name(&storage_name)
        .await
        .map_err(InternalError::from)?
        .ok_or_else(|| APIError::from(("Storage not found", StatusCode::NOT_FOUND)))?;
    let repository = storage
        .get_repository(&repository_name)
        .await
        .map_err(InternalError::from)?
        .ok_or_else(|| APIError::from(("Repository not found", StatusCode::NOT_FOUND)))?;
    storage
        .delete_repository(&repository, query_params.purge_repository)
        .await
        .map_err(InternalError::from)?;
    Ok(HttpResponse::NoContent().finish())
}

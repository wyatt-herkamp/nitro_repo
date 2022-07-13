use actix_web::http::StatusCode;
use actix_web::{delete, get, post, web, HttpResponse, ResponseError};
use sea_orm::DatabaseConnection;

use crate::authentication::Authentication;
use crate::error::api_error::APIError;
use crate::error::internal_error::InternalError;
use crate::storage::error::StorageError;
use crate::storage::models::Storage;
use crate::storage::models::StorageFactory;
use crate::storage::multi::MultiStorageController;
use crate::system::permissions::options::CanIDo;
use crate::system::user::UserModel;

#[utoipa::path(
get,
path = "/api/storages",
responses((status = 200, description = "A list of storages", body = [StorageSaver])),
)]
#[get("/storages")]
pub async fn get_storages(
    storage_handler: web::Data<MultiStorageController>,
    database: web::Data<DatabaseConnection>,
    auth: Authentication,
) -> actix_web::Result<HttpResponse> {
    let user: UserModel = auth.get_user(&database).await??;
    user.can_i_edit_repos()?;

    Ok(HttpResponse::Ok().json(storage_handler.storage_savers().await))
}

/// Creates a new storage based on the Storage Factory
#[post("/storage/new")]
pub async fn new_storage(
    storage_handler: web::Data<MultiStorageController>,
    new_storage: web::Json<StorageFactory>,
    database: web::Data<DatabaseConnection>,
    auth: Authentication,
) -> actix_web::Result<HttpResponse> {
    let user: UserModel = auth.get_user(&database).await??;
    user.can_i_edit_repos()?;
    if let Err(error) = storage_handler.create_storage(new_storage.0).await {
        match error {
            StorageError::StorageAlreadyExist => Ok(APIError::from((
                "Storage already exist",
                StatusCode::CONFLICT,
            ))
            .error_response()),
            _ => Err(InternalError::from(error).into()),
        }
    } else {
        Ok(HttpResponse::Ok().finish())
    }
}

/// Delete the storage based on the name
#[delete("/storage/{name}")]
pub async fn delete_storage(
    storage_handler: web::Data<MultiStorageController>,
    database: web::Data<DatabaseConnection>,
    auth: Authentication,
    name: web::Path<String>,
) -> actix_web::Result<HttpResponse> {
    let user: UserModel = auth.get_user(&database).await??;
    user.can_i_edit_repos()?;
    if storage_handler
        .delete_storage(&name.into_inner())
        .await
        .map_err(InternalError::from)?
    {
        Ok(HttpResponse::Ok().finish())
    } else {
        Ok(APIError::from(("Storage does not exist", StatusCode::NOT_FOUND)).error_response())
    }
}

#[utoipa::path(
get,
path = "/api/storages/{id}",
responses(
(status = 200, description = "Storage found succesfully", body = StorageSaver),
(status = 404, description = "Storage was not found")
),
params(
("id" = string, path, description = "Storage name"),
)
)]
#[get("/storage/{name}")]
pub async fn get_storage(
    storage_handler: web::Data<MultiStorageController>,
    name: web::Path<String>,
    database: web::Data<DatabaseConnection>,
    auth: Authentication,
) -> actix_web::Result<HttpResponse> {
    let user: UserModel = auth.get_user(&database).await??;
    user.can_i_edit_repos()?;
    if let Some(storage) = storage_handler
        .get_storage_by_name(&name.into_inner())
        .await
        .map_err(InternalError::from)?
    {
        Ok(HttpResponse::Ok().json(storage.config_for_saving()))
    } else {
        Ok(APIError::from(("Storage not found", StatusCode::NOT_FOUND)).error_response())
    }
}

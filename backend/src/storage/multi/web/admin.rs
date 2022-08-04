use actix_web::http::StatusCode;
use actix_web::{delete, get, post, web, HttpResponse, ResponseError};
use sea_orm::DatabaseConnection;

use crate::authentication::Authentication;
use crate::error::internal_error::InternalError;
use crate::storage::error::StorageError;
use crate::storage::models::Storage;
use crate::storage::multi::{MultiStorageController, PurgeLevel};
use crate::storage::{DynamicStorage, StorageSaver};
use crate::system::permissions::options::CanIDo;

#[get("/storages")]
pub async fn get_storages(
    storage_handler: web::Data<MultiStorageController<DynamicStorage>>,
    database: web::Data<DatabaseConnection>,
    auth: Authentication,
) -> actix_web::Result<HttpResponse> {
    let user = auth.get_user(&database).await??;
    user.can_i_edit_repos()?;

    Ok(HttpResponse::Ok().json(storage_handler.storage_savers().await))
}

/// Creates a new storage based on the Storage Factory
#[post("/storage/new")]
pub async fn new_storage(
    storage_handler: web::Data<MultiStorageController<DynamicStorage>>,
    new_storage: web::Json<StorageSaver>,
    database: web::Data<DatabaseConnection>,
    auth: Authentication,
) -> actix_web::Result<HttpResponse> {
    let user = auth.get_user(&database).await??;
    user.can_i_edit_repos()?;
    if let Err(error) = storage_handler.create_storage(new_storage.0).await {
        match error {
            StorageError::StorageAlreadyExist => Ok(HttpResponse::Conflict().finish()),
            _ => Err(InternalError::from(error).into()),
        }
    } else {
        Ok(HttpResponse::Ok().finish())
    }
}

/// Delete the storage based on the name
#[delete("/storage/{name}/{level}")]
pub async fn delete_storage(
    storage_handler: web::Data<MultiStorageController<DynamicStorage>>,
    database: web::Data<DatabaseConnection>,
    auth: Authentication,
    request: web::Path<(String, PurgeLevel)>,
) -> actix_web::Result<HttpResponse> {
    let user = auth.get_user(&database).await??;
    user.can_i_edit_repos()?;
    let (name, level) = request.into_inner();
    storage_handler
        .delete_storage(&name, level)
        .await
        .map_err(InternalError::from)?;
    Ok(HttpResponse::NoContent().finish())
}

#[get("/storage/{name}")]
pub async fn get_storage(
    storage_handler: web::Data<MultiStorageController<DynamicStorage>>,
    name: web::Path<String>,
    database: web::Data<DatabaseConnection>,
    auth: Authentication,
) -> actix_web::Result<HttpResponse> {
    let user = auth.get_user(&database).await??;
    user.can_i_edit_repos()?;
    let storage = crate::helpers::get_storage!(storage_handler, name);
    Ok(HttpResponse::Ok().json(storage.storage_config()))
}

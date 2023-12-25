use actix_web::{delete, get, post, web, HttpResponse};
use sea_orm::DatabaseConnection;

use crate::{
    authentication::{Authentication, TrulyAuthenticated},
    error::internal_error::InternalError,
    storage::{
        error::StorageError,
        models::Storage,
        multi::{MultiStorageController, PurgeLevel},
        DynamicStorage, StorageSaver,
    },
    system::permissions::permissions_checker::CanIDo,
};

#[get("/storages")]
pub async fn get_storages(
    storage_handler: web::Data<MultiStorageController<DynamicStorage>>,
    database: web::Data<DatabaseConnection>,
    auth: TrulyAuthenticated,
) -> actix_web::Result<HttpResponse> {
    let user = auth.into_user();
    user.can_i_edit_repos()?;

    Ok(HttpResponse::Ok().json(storage_handler.storage_savers().await))
}

/// Creates a new storage based on the Storage Factory
#[post("/storage/new")]
pub async fn new_storage(
    storage_handler: web::Data<MultiStorageController<DynamicStorage>>,
    new_storage: web::Json<StorageSaver>,
    database: web::Data<DatabaseConnection>,
    auth: TrulyAuthenticated,
) -> actix_web::Result<HttpResponse> {
    let user = auth.into_user();
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
    auth: TrulyAuthenticated,
    request: web::Path<(String, PurgeLevel)>,
) -> actix_web::Result<HttpResponse> {
    let user = auth.into_user();
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
    auth: TrulyAuthenticated,
) -> actix_web::Result<HttpResponse> {
    let user = auth.into_user();
    user.can_i_edit_repos()?;
    let storage = name.into_inner();
    let storage = crate::helpers::get_storage!(storage_handler, storage);
    Ok(HttpResponse::Ok().json(storage.storage_config()))
}

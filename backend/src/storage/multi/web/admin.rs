use actix_web::http::StatusCode;
use actix_web::{get, post, web, HttpResponse, ResponseError};
use sea_orm::DatabaseConnection;

use crate::authentication::Authentication;
use crate::error::api_error::APIError;
use crate::error::internal_error::InternalError;
use crate::storage::error::StorageError;
use crate::storage::models::StorageFactory;
use crate::storage::multi::MultiStorageController;
use crate::system::permissions::options::CanIDo;
use crate::system::user::UserModel;

/// API handler for getting storages
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

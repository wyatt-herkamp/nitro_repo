use actix_web::http::StatusCode;
use actix_web::{delete, get, post, web, HttpResponse};
use serde::Serialize;

use crate::api_response::{APIResponse, NRResponse};

use crate::system::permissions::options::CanIDo;

use sea_orm::DatabaseConnection;

use crate::authentication::Authentication;

use crate::storage::models::StorageFactory;
use crate::storage::multi::MultiStorageController;
use crate::system::user::UserModel;

#[get("/api/storages/list")]
pub async fn list_storages(
    connection: web::Data<DatabaseConnection>,
    storages: web::Data<MultiStorageController>,
    auth: Authentication,
) -> NRResponse {
    let caller: UserModel = auth.get_user(&connection).await??;
    caller.can_i_edit_repos()?;

    APIResponse::from(Some(
        serde_json::to_value(&storages.storage_savers().await).unwrap(),
    ))
}

#[delete("/api/admin/storages/{id}")]
pub async fn delete_by_id(
    connection: web::Data<DatabaseConnection>,
    auth: Authentication,
    storages: web::Data<MultiStorageController>,
    id: web::Path<String>,
) -> NRResponse {
    let caller: UserModel = auth.get_user(&connection).await??;
    caller.can_i_edit_repos()?;
    if storages.delete_storage(id.as_str()).await? {
        Ok(APIResponse::ok())
    } else {
        Ok(StatusCode::NOT_FOUND.into())
    }
}

#[get("/api/storages/id/{id}")]
pub async fn get_by_id(
    connection: web::Data<DatabaseConnection>,
    auth: Authentication,
    storages: web::Data<MultiStorageController>,
    id: web::Path<String>,
) -> NRResponse {
    let caller: UserModel = auth.get_user(&connection).await??;
    caller.can_i_edit_repos()?;

    let option = storages.get_storage_by_name(id.as_ref()).await?;
    if option.is_none() {
        Ok(StatusCode::NOT_FOUND.into())
    }
    let storage = option.unwrap();
    Ok(APIResponse::from(Some(storage.config_for_saving())))
}

#[post("/api/admin/storages/add")]
pub async fn add_storage(
    connection: web::Data<DatabaseConnection>,
    auth: Authentication,
    storage: web::Json<StorageFactory>,
    storages: web::Data<MultiStorageController>,
) -> NRResponse {
    let caller: UserModel = auth.get_user(&connection).await??;
    caller.can_i_edit_repos()?;
    storages.create_storage(storage.0).await?;
    Ok(APIResponse::ok())
}

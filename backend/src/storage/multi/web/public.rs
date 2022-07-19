use actix_web::{get, web, HttpResponse};
use sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};
use utoipa::Component;

use crate::authentication::Authentication;
use crate::storage::models::Storage;
use crate::storage::multi::MultiStorageController;
#[derive(Debug, Clone, Serialize, Deserialize, Component)]
pub struct PublicStorageResponse {
    /// The List of Storages that are available to the current user
    pub storages: Vec<String>,
    /// If the system is a multi or single storage system. If False storages will contain only one element named System
    pub multi_storage: bool,
}
#[utoipa::path(
get,
path = "/api/storages",
responses((status = 200, description = "A list of storages", body = PublicStorageResponse)),

)]
#[get("/storages")]
pub async fn get_storages_multi(
    storage_handler: web::Data<MultiStorageController>,
    database: web::Data<DatabaseConnection>,
    auth: Authentication,
) -> actix_web::Result<HttpResponse> {
    let names = storage_handler.names().await;
    let public_storage_response = PublicStorageResponse {
        storages: names,
        multi_storage: true,
    };
    Ok(HttpResponse::Ok().json(public_storage_response))
}
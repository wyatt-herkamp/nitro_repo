use actix_web::{get, web, HttpResponse};
use sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLockReadGuard;
use utoipa::Component;

use crate::authentication::Authentication;
use crate::helpers::unwrap_or_not_found;
use crate::repository::settings::RepositoryType;
use crate::storage::models::Storage;
use crate::storage::multi::MultiStorageController;
use crate::storage::DynamicStorage;
#[derive(Debug, Clone, Serialize, Deserialize, Component)]
pub struct PublicRepositoryResponse {
    pub name: String,
    pub repository_type: RepositoryType,
}
#[utoipa::path(
get,
path = "/api/repositories/{storage_name}",
responses((status = 200, description = "A list of storages", body = [PublicRepositoryResponse])),

)]
#[get("repositories/{storage_name}")]
pub async fn get_repositories(
    storage_handler: web::Data<MultiStorageController>,
    database: web::Data<DatabaseConnection>,
    auth: Authentication,
    path: web::Path<String>,
) -> actix_web::Result<HttpResponse> {
    let storage_name = path.into_inner();
    let value: RwLockReadGuard<'_, DynamicStorage> = unwrap_or_not_found!(storage_handler
        .get_storage_by_name(&storage_name)
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?);

    let vec: Vec<PublicRepositoryResponse> = value
        .get_repositories()
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?
        .into_iter()
        .map(|r| PublicRepositoryResponse {
            name: r.name,
            repository_type: r.repository_type,
        })
        .collect::<Vec<PublicRepositoryResponse>>();

    Ok(HttpResponse::Ok().json(vec))
}

use actix_web::{
    get, post,
    web::{self, ServiceConfig},
    HttpResponse,
};
use nr_core::database::storage::DBStorage;
use nr_storage::{StorageConfig, StorageFactory, StorageTypeConfig};
use serde::{Deserialize, Serialize};
use sqlx::types::Json;
use tracing::error;

use crate::{
    app::{DatabaseConnection, NitroRepo},
    error::internal_error::InternalError,
};

pub fn init(service: &mut ServiceConfig) {
    service.service(list_storages).service(new_storage);
}

#[get("/list")]
pub async fn list_storages(database: DatabaseConnection) -> Result<HttpResponse, InternalError> {
    let storages = DBStorage::get_all(&database).await?;
    Ok(HttpResponse::Ok().json(storages))
}
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NewStorageRequest {
    pub name: String,
    pub config: StorageTypeConfig,
}
#[post("/new/{storage_type}")]
pub async fn new_storage(
    database: DatabaseConnection,
    storage_type: web::Path<String>,
    request: web::Json<NewStorageRequest>,
    site: web::Data<NitroRepo>,
) -> Result<HttpResponse, InternalError> {
    //TODO Permissions
    let storage_type = storage_type.into_inner();
    let request = request.into_inner();
    if !DBStorage::is_name_available(&request.name, database.as_ref()).await? {
        return Ok(HttpResponse::Conflict().finish());
    }

    let Some(storage_factory) = site.get_storage_factory(&storage_type) else {
        return Ok(HttpResponse::BadRequest().finish());
    };
    if let Err(error) = storage_factory
        .test_storage_config(request.config.clone())
        .await
    {
        return Ok(HttpResponse::BadRequest().body(error.to_string()));
    }
    let id = DBStorage::generate_uuid(database.as_ref()).await?;
    let config = serde_json::to_value(request.config).unwrap();
    let storage = DBStorage {
        id,
        storage_type,
        name: request.name,
        config: Json(config),
        active: true,
        created: chrono::Utc::now().fixed_offset(),
    };
    let storage = storage.insert(&database).await?;
    //TODO on Error revert the database
    let storage_config = match StorageConfig::try_from(storage.clone()) {
        Ok(ok) => ok,
        Err(err) => {
            DBStorage::delete(id, database.as_ref()).await?;
            error!("Failed to create storage config: {}", err);
            return Ok(HttpResponse::InternalServerError().finish());
        }
    };
    match storage_factory.create_storage(storage_config).await {
        Ok(ok) => site.add_storage(id, ok),
        Err(err) => {
            DBStorage::delete(id, database.as_ref()).await?;
            error!("Failed to create storage: {}", err);
            return Ok(HttpResponse::InternalServerError().finish());
        }
    }

    Ok(HttpResponse::Created().json(storage))
}

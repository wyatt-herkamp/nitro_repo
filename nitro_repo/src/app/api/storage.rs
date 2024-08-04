use axum::{
    body::Body,
    extract::{Path, State},
    response::{IntoResponse, Response},
    Json,
};
use http::StatusCode;
use nr_core::database::storage::DBStorage;
use nr_storage::{StorageConfig, StorageFactory, StorageTypeConfig};
use serde::{Deserialize, Serialize};
use tracing::{error, instrument};
use uuid::Uuid;

use crate::{app::NitroRepo, error::internal_error::InternalError};
pub fn storage_routes() -> axum::Router<crate::app::api::storage::NitroRepo> {
    axum::Router::new()
        .route("/list", axum::routing::get(list_storages))
        .route("/new", axum::routing::post(new_storage))
        .route("/:id", axum::routing::get(get_storage))
}
#[instrument]

pub async fn list_storages(State(site): State<NitroRepo>) -> Result<Response, InternalError> {
    let storages = DBStorage::get_all(&site.database).await?;
    Ok(Response::builder()
        .status(200)
        .header("Content-Type", "application/json")
        .body(Body::from(serde_json::to_string(&storages).unwrap()))
        .unwrap())
}
#[derive(Debug)]
pub enum NewStorageResponse {
    Created(DBStorage),
    InvalidConfig,
    InvalidType,
    NameInUse,
    InternalError,
}
impl IntoResponse for NewStorageResponse {
    fn into_response(self) -> Response {
        match self {
            NewStorageResponse::Created(storage) => Response::builder()
                .status(StatusCode::CREATED)
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_string(&storage).unwrap()))
                .unwrap(),
            NewStorageResponse::InvalidConfig => Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .body("Invalid Storage Config".into())
                .unwrap(),
            NewStorageResponse::InvalidType => Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .body("Invalid Storage Type".into())
                .unwrap(),
            NewStorageResponse::NameInUse => Response::builder()
                .status(StatusCode::CONFLICT)
                .body("Name already in use".into())
                .unwrap(),
            NewStorageResponse::InternalError => Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body("Internal Server Error".into())
                .unwrap(),
        }
    }
}
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NewStorageRequest {
    pub name: String,
    pub config: StorageTypeConfig,
}
#[instrument]

pub async fn new_storage(
    State(site): State<NitroRepo>,
    Path(storage_type): Path<String>,
    Json(request): Json<NewStorageRequest>,
) -> Result<NewStorageResponse, InternalError> {
    //TODO Permissions

    if !DBStorage::is_name_available(&request.name, site.as_ref()).await? {
        return Ok(NewStorageResponse::NameInUse);
    }

    let Some(storage_factory) = site.get_storage_factory(&storage_type) else {
        return Ok(NewStorageResponse::InvalidType);
    };
    if let Err(error) = storage_factory
        .test_storage_config(request.config.clone())
        .await
    {
        error!("Failed to test storage config: {}", error);
        return Ok(NewStorageResponse::InvalidConfig);
    }
    let id = DBStorage::generate_uuid(site.as_ref()).await?;
    let config = serde_json::to_value(request.config).unwrap();
    let storage = DBStorage {
        id,
        storage_type,
        name: request.name,
        config: sqlx::types::Json(config),
        active: true,
        created: chrono::Utc::now().fixed_offset(),
    };
    let storage = storage.insert(&site.database).await?;
    //TODO on Error revert the database
    let storage_config = match StorageConfig::try_from(storage.clone()) {
        Ok(ok) => ok,
        Err(err) => {
            DBStorage::delete(id, site.as_ref()).await?;
            error!("Failed to create storage config: {}", err);
            return Ok(NewStorageResponse::InternalError);
        }
    };
    match storage_factory.create_storage(storage_config).await {
        Ok(ok) => site.add_storage(id, ok),
        Err(err) => {
            DBStorage::delete(id, site.as_ref()).await?;
            error!("Failed to create storage: {}", err);
            return Ok(NewStorageResponse::InternalError);
        }
    }

    Ok(NewStorageResponse::Created(storage))
}
#[instrument]

pub async fn get_storage(
    Path(id): Path<Uuid>,
    State(site): State<NitroRepo>,
) -> Result<Response, InternalError> {
    let storage = DBStorage::get(id, &site.database).await?;
    match storage {
        Some(storage) => Ok(Response::builder()
            .status(200)
            .header("Content-Type", "application/json")
            .body(Body::from(serde_json::to_string(&storage).unwrap()))
            .unwrap()),
        None => Ok(Response::builder()
            .status(404)
            .body("Storage not found".into())
            .unwrap()),
    }
}

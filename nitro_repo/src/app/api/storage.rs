use axum::{
    body::Body,
    extract::{Path, State},
    response::{IntoResponse, Response},
    Json,
};
use http::{header::CONTENT_TYPE, StatusCode};
use nr_core::{
    database::storage::{DBStorage, NewDBStorage},
    user::permissions::HasPermissions,
};
use nr_storage::{local::LocalConfig, StorageConfig, StorageFactory, StorageTypeConfig};
use serde::{Deserialize, Serialize};
use tracing::{error, instrument};
use utoipa::{OpenApi, ToSchema};
use uuid::Uuid;

use crate::{
    app::{
        authentication::Authentication,
        responses::{InvalidStorageConfig, InvalidStorageType, MissingPermission},
        NitroRepo,
    },
    error::InternalError,
};
#[derive(OpenApi)]
#[openapi(
    paths(list_storages, new_storage, get_storage),
    components(schemas(DBStorage, NewStorageRequest, StorageTypeConfig, LocalConfig))
)]
pub struct StorageAPI;
pub fn storage_routes() -> axum::Router<crate::app::api::storage::NitroRepo> {
    axum::Router::new()
        .route("/list", axum::routing::get(list_storages))
        .route("/new/:storage_type", axum::routing::post(new_storage))
        .route("/:id", axum::routing::get(get_storage))
}

#[utoipa::path(
    get,
    path = "/list",
    responses(
        (status = 200, description = "information about the Site", body = Instance)
    )
)]
#[instrument]
pub async fn list_storages(
    State(site): State<NitroRepo>,
    auth: Authentication,
) -> Result<Response, InternalError> {
    if !auth.is_admin_or_storage_manager() {
        return Ok(MissingPermission::StorageManager.into_response());
    }
    let storages = DBStorage::get_all(&site.database).await?;
    Ok(Response::builder()
        .status(200)
        .header("Content-Type", "application/json")
        .body(Body::from(serde_json::to_string(&storages).unwrap()))
        .unwrap())
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct NewStorageRequest {
    pub name: String,
    pub config: StorageTypeConfig,
}

#[utoipa::path(
    post,
    path = "/new/{storage_type}",
    request_body = NewStorageRequest,
    responses(
        (status = 200, description = "information about the Site", body = DBStorage),
        (status = 409, description = "Name already in use"),
        (status = 400, description = "Invalid Storage Config"),
    ),
    params(
        ("storage_type" = String, Path, description = "Storage Type"),
    )
)]
#[instrument]
pub async fn new_storage(
    auth: Authentication,
    State(site): State<NitroRepo>,
    Path(storage_type): Path<String>,
    Json(request): Json<NewStorageRequest>,
) -> Result<Response, InternalError> {
    if !auth.is_admin_or_storage_manager() {
        return Ok(MissingPermission::StorageManager.into_response());
    }
    if !DBStorage::is_name_available(&request.name, site.as_ref()).await? {
        return Ok(Response::builder()
            .status(StatusCode::CONFLICT)
            .body("Name already in use".into())
            .unwrap());
    }

    let Some(storage_factory) = site.get_storage_factory(&storage_type) else {
        return Ok(InvalidStorageType(storage_type).into_response());
    };
    if let Err(error) = storage_factory
        .test_storage_config(request.config.clone())
        .await
    {
        error!("Failed to test storage config: {}", error);
        return Ok(InvalidStorageConfig(error).into_response());
    }
    let config = serde_json::to_value(request.config).unwrap();
    let storage = NewDBStorage::new(request.name, storage_type, config)
        .insert(&site.database)
        .await?;
    let Some(storage) = storage else {
        return Ok(Response::builder()
            .status(StatusCode::CONFLICT)
            .body("Name already in use".into())
            .unwrap());
    };
    let id = storage.id;
    let storage_config = match StorageConfig::try_from(storage.clone()) {
        Ok(ok) => ok,
        Err(err) => {
            DBStorage::delete(id, site.as_ref()).await?;
            error!("Failed to create storage config: {}", err);
            return Err(InternalError::from(err));
        }
    };
    match storage_factory.create_storage(storage_config).await {
        Ok(ok) => site.add_storage(id, ok),
        Err(err) => {
            DBStorage::delete(id, site.as_ref()).await?;
            error!("Failed to create storage: {}", err);
            return Err(InternalError::from(err));
        }
    }
    Ok(Response::builder()
        .status(StatusCode::CREATED)
        .header(CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::to_string(&storage).unwrap()))
        .unwrap())
}
#[utoipa::path(
    post,
    path = "/{id}",
    responses(
        (status = 200, description = "Storage Configuration", body = DBStorage),
        (status = 404, description = "Storage not found")
    )
)]
#[instrument]
pub async fn get_storage(
    auth: Authentication,

    Path(id): Path<Uuid>,
    State(site): State<NitroRepo>,
) -> Result<Response, InternalError> {
    if !auth.is_admin_or_storage_manager() {
        return Ok(MissingPermission::StorageManager.into_response());
    }
    let storage = DBStorage::get(id, &site.database).await?;
    match storage {
        Some(storage) => {
            let response = Json(storage).into_response();
            Ok(response)
        }
        None => Ok(Response::builder()
            .status(404)
            .body("Storage not found".into())
            .unwrap()),
    }
}

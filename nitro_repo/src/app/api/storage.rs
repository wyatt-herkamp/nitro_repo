use axum::{
    Json,
    extract::{Path, Query, State},
    response::{IntoResponse, Response},
    routing::{get, post, put},
};
use nr_core::{
    database::entities::storage::{DBStorage, DBStorageNoConfig, NewDBStorage, StorageDBType},
    storage::StorageName,
    user::{permissions::HasPermissions, scopes::NRScope},
};
use nr_storage::{
    Storage, StorageConfig, StorageTypeConfig, fs_v2::FileSystemV2Config, local::LocalConfig,
};
use serde::{Deserialize, Serialize};
use tracing::{error, instrument};
use utoipa::{IntoParams, OpenApi, ToSchema};
use uuid::Uuid;
mod local;
mod s3;
use nr_web_core::{
    authentication::Authentication,
    error::InternalError,
    responses::{InvalidStorageConfig, InvalidStorageType, MissingPermission},
    utils::{ResponseBuilder, conflict::ConflictResponse},
};

use crate::app::{NitroRepo, api::require_scope};
#[derive(OpenApi)]
#[openapi(
    paths(list_storages, new_storage, get_storage, update_storage),
    components(schemas(
        DBStorage,
        NewStorageRequest,
        UpdateStorageRequest,
        StorageTypeConfig,
        LocalConfig,
        FileSystemV2Config
    )),
    nest(
        (path = "/local", api = local::LocalStorageAPI, tags=["local", "storage"]),
        (path = "/s3", api = s3::S3StorageAPI, tags=["s3", "storage"])
    ),
    tags(
        (name= "local", description = "Local Storage"),
        (name= "s3", description = "S3 Storage"),
    )
)]
pub struct StorageAPI;
pub fn storage_routes() -> axum::Router<crate::app::api::storage::NitroRepo> {
    axum::Router::new()
        .route("/list", get(list_storages))
        .route("/new/{storage_type}", post(new_storage))
        .route("/{id}", get(get_storage))
        .route("/{id}", put(update_storage))
        .nest("/local", local::local_storage_routes())
        .nest("/s3", s3::s3_storage_api())
}
#[derive(Debug, Default, Serialize, Deserialize, ToSchema, IntoParams)]
#[serde(default)]
#[into_params(parameter_in = Query)]
pub struct StorageListRequest {
    /// Include the storage configuration in the response (default: false)
    pub include_config: bool,
    /// Only include active storages (default: false)
    pub active_only: bool,
}
#[utoipa::path(
    get,
    path = "/list",
    params(
        StorageListRequest,
    ),
    responses(
        (status = 200, description = "All Storages registered to the system.", body = [DBStorage]),
        (status = 200, description = "All the storages without the configs", body = [DBStorageNoConfig]),
        (status = 403, description = "Does not have permission to view storages")
    )
)]
#[instrument]
pub async fn list_storages(
    State(site): State<NitroRepo>,
    auth: Authentication,
    Query(request): Query<StorageListRequest>,
) -> Result<Response, InternalError> {
    if !auth.is_admin_or_system_manager() {
        return Ok(MissingPermission::StorageManager.into_response());
    }
    if let Some(denied) = require_scope(&auth, NRScope::ReadStorage, &site).await? {
        return Ok(denied);
    }
    if request.include_config {
        let storages = DBStorage::get_all(&site.database).await?;
        Ok(ResponseBuilder::ok().json(&storages))
    } else {
        let storages = DBStorageNoConfig::get_all(&site.database).await?;
        Ok(ResponseBuilder::ok().json(&storages))
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct NewStorageRequest {
    pub name: StorageName,
    pub config: StorageTypeConfig,
}

#[utoipa::path(
    post,
    path = "/new/{storage_type}",
    request_body = NewStorageRequest,
    responses(
        (status = 201, description = "Storage Successfully Created", body = DBStorage),
        ConflictResponse,
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
    if !auth.is_admin_or_system_manager() {
        return Ok(MissingPermission::StorageManager.into_response());
    }
    if let Some(denied) = require_scope(&auth, NRScope::ManageStorage, &site).await? {
        return Ok(denied);
    }
    if !DBStorage::is_name_available(&request.name, site.as_ref()).await? {
        return Ok(ConflictResponse::from("name").into_response());
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
    let storage = NewDBStorage::new(storage_type, request.name, config)
        .insert(&site.database)
        .await?;
    let Some(storage) = storage else {
        return Ok(ConflictResponse::from("name").into_response());
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
    Ok(ResponseBuilder::created().json(&storage))
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
    if !auth.is_admin_or_system_manager() {
        return Ok(MissingPermission::StorageManager.into_response());
    }
    if let Some(denied) = require_scope(&auth, NRScope::ReadStorage, &site).await? {
        return Ok(denied);
    }
    let storage = DBStorage::get_by_id(id, &site.database).await?;
    match storage {
        Some(storage) => Ok(ResponseBuilder::ok().json(&storage)),
        None => Ok(ResponseBuilder::not_found().body("Storage not found")),
    }
}

#[derive(Debug, Default, Serialize, Deserialize, ToSchema)]
#[serde(default)]
pub struct UpdateStorageRequest {
    pub name: Option<StorageName>,
    /// Replaces the whole configuration. Omit it to leave the stored one alone.
    pub config: Option<StorageTypeConfig>,
    pub active: Option<bool>,
}

#[utoipa::path(
    put,
    path = "/{id}",
    request_body = UpdateStorageRequest,
    responses(
        (status = 200, description = "The updated storage", body = DBStorage),
        (status = 404, description = "Storage not found"),
        (status = 409, description = "Another storage already has that name"),
    )
)]
#[instrument(skip(site))]
pub async fn update_storage(
    auth: Authentication,
    State(site): State<NitroRepo>,
    Path(id): Path<Uuid>,
    Json(request): Json<UpdateStorageRequest>,
) -> Result<Response, InternalError> {
    if !auth.is_admin_or_system_manager() {
        return Ok(MissingPermission::StorageManager.into_response());
    }
    if let Some(denied) = require_scope(&auth, NRScope::ManageStorage, &site).await? {
        return Ok(denied);
    }

    let Some(existing) = DBStorage::get_by_id(id, site.as_ref()).await? else {
        return Ok(ResponseBuilder::not_found().body("Storage not found"));
    };

    // Checked before writing so a rename reports a conflict rather than a constraint violation.
    // The unique index is still what makes it correct under a race.
    if let Some(name) = &request.name
        && name != &existing.name
        && DBStorage::is_name_taken_by_other(id, name, site.as_ref()).await?
    {
        return Ok(ConflictResponse::from("name").into_response());
    }

    // A configuration change is offered to the running storage before it is persisted: a backend
    // that cannot move to the new config (an unreachable bucket, a path it cannot write) should
    // fail here rather than leave a row that will not load on the next restart.
    if let Some(config) = &request.config {
        let Some(storage) = site.get_storage(id) else {
            return Ok(ResponseBuilder::not_found().body("Storage is not loaded"));
        };
        if let Err(error) = storage.validate_config_change(config.clone()).await {
            error!(?error, "Rejected a storage config change");
            return Ok(InvalidStorageConfig(error).into_response());
        }
    }

    let config = request
        .config
        .map(serde_json::to_value)
        .transpose()
        .map_err(InternalError::from)?;

    let updated = DBStorage::update_details(
        id,
        request.name.as_ref(),
        config.as_ref(),
        request.active,
        site.as_ref(),
    )
    .await?;

    let Some(updated) = updated else {
        return Ok(ResponseBuilder::not_found().body("Storage not found"));
    };

    Ok(ResponseBuilder::ok().json(&updated))
}

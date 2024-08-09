use ahash::HashMap;
use axum::{
    body::Body,
    debug_handler,
    extract::{Path, State},
    response::{IntoResponse, Response},
    Json,
};
use http::{header::CONTENT_TYPE, StatusCode};
use nr_core::{
    database::repository::{DBRepository, DBRepositoryWithStorageName, GenericDBRepositoryConfig},
    user::permissions::HasPermissions,
};
use serde::Deserialize;
use serde_json::Value;
use tracing::{error, instrument};
use utoipa::{OpenApi, ToSchema};
use uuid::Uuid;

use crate::{
    app::{
        authentication::Authentication,
        responses::{InvalidRepositoryConfig, MissingPermission, RepositoryNotFound},
        NitroRepo,
    },
    error::InternalError,
    repository::{Repository, RepositoryTypeDescription},
};
mod config;

#[derive(OpenApi)]
#[openapi(
    paths(
        repository_types,
        list_repositories,
        config::config_schema,
        config::config_validate,
        config::config_default,
        new_repository
    ),
    components(schemas(DBRepository, DBRepositoryWithStorageName, RepositoryTypeDescription))
)]
pub struct RepositoryAPI;
pub fn repository_routes() -> axum::Router<NitroRepo> {
    axum::Router::new()
        .route("/list", axum::routing::get(list_repositories))
        .route("/new/:repository_type", axum::routing::post(new_repository))
        .route("/:id/config/:key", axum::routing::put(update_config))
        .route("/:id/config/:key", axum::routing::get(get_repository))
        .route("/types", axum::routing::get(repository_types))
        .merge(config::config_routes())
}

#[utoipa::path(
    get,
    path = "/types",
    responses(
        (status = 200, description = "Repository Types", body = [RepositoryTypeDescription]),
    )
)]
#[instrument]
pub async fn repository_types(State(site): State<NitroRepo>) -> Response {
    // TODO: Add Client side caching

    let types: Vec<_> = site
        .repository_types
        .iter()
        .map(|v| v.get_description())
        .collect();
    Json(types).into_response()
}
#[instrument]
pub async fn get_repository(
    State(site): State<NitroRepo>,
    auth: Authentication,
    Path((repository, config_key)): Path<(Uuid, String)>,
) -> Result<Response, InternalError> {
    if !auth.can_edit_repository(repository) {
        return Ok(MissingPermission::EditRepository(repository).into_response());
    }
    let Some(config) =
        GenericDBRepositoryConfig::get_config(repository, config_key, site.as_ref()).await?
    else {
        // TODO: Check for default config
        return Ok(Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body("".into())
            .unwrap());
    };
    let response = Json(config).into_response();
    Ok(response)
}

#[instrument]
pub async fn update_config(
    State(site): State<NitroRepo>,
    auth: Authentication,
    Path((repository, config_key)): Path<(Uuid, String)>,
    Json(config): Json<serde_json::Value>,
) -> Result<Response, InternalError> {
    if !auth.can_edit_repository(repository) {
        return Ok(MissingPermission::EditRepository(repository).into_response());
    }
    let Some(config_type) = site.get_repository_config_type(&config_key) else {
        return Ok(InvalidRepositoryConfig::InvalidConfigType(config_key).into_response());
    };
    let Some(db_repository) = DBRepository::get_by_id(repository, site.as_ref()).await? else {
        return Ok(RepositoryNotFound::Uuid(repository).into_response());
    };
    let Some(repository) = site.get_repository(db_repository.id) else {
        return Ok(Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body("Repository Exists. But it is not loaded. Illegal State".into())
            .unwrap());
    };
    if !repository.config_types().contains(&config_key.as_str()) {
        let repository = repository.base_config();
        return Ok(InvalidRepositoryConfig::RepositoryTypeDoesntSupportConfig {
            repository_type: repository.repository_type,
            config_key,
        }
        .into_response());
    }
    if let Err(error) = config_type.validate_config(config.clone()) {
        error!("Error validating config: {}", error);
        return Ok(InvalidRepositoryConfig::InvalidConfig { config_key, error }.into_response());
    }
    GenericDBRepositoryConfig::add_or_update(db_repository.id, config_key, config, site.as_ref())
        .await?;
    if let Err(err) = repository.reload().await {
        return Ok(Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(format!("Failed to reload repository: {}", err).into())
            .unwrap());
    }
    Ok(Response::builder()
        .status(StatusCode::OK)
        .body(Body::empty())
        .unwrap())
}

#[utoipa::path(
    get,
    path = "/list",
    responses(
        (status = 200, description = "List Repositories", body = [DBRepositoryWithStorageName]),
    )
)]
#[instrument]
pub async fn list_repositories(
    auth: Authentication,
    State(site): State<NitroRepo>,
) -> Result<Response, InternalError> {
    let repositories: Vec<_> = DBRepositoryWithStorageName::get_all(site.as_ref())
        .await?
        .into_iter()
        .filter(|repo| auth.can_edit_repository(repo.id))
        .collect();
    let response = Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .body(serde_json::to_string(&repositories).unwrap().into())
        .unwrap();
    Ok(response)
}
#[derive(Deserialize, ToSchema, Debug)]
pub struct NewRepositoryRequest {
    /// The Name of the Repository
    pub name: String,
    /// The Storage ID
    pub storage: Uuid,
    /// Optional Sub Type of the Repository
    /// A Map of Config Key to Config Value
    pub configs: HashMap<String, Value>,
}
#[utoipa::path(
    post,
    request_body = NewRepositoryRequest,
    path = "/new/{repository_type}",
    responses(
        (status = 200, description = "Create new Repository", body = DBRepository),
    )
)]
#[debug_handler]
pub async fn new_repository(
    State(site): State<NitroRepo>,
    auth: Authentication,
    Path(repository_type): Path<String>,
    Json(request): Json<NewRepositoryRequest>,
) -> Result<Response, InternalError> {
    if !auth.is_admin_or_repository_manager() {
        return Ok(MissingPermission::RepositoryManager.into_response());
    }
    let NewRepositoryRequest {
        name,
        configs,
        storage,
    } = request;
    let Some(repository_factory) = site.get_repository_type(&repository_type) else {
        return Ok(InvalidRepositoryConfig::InvalidConfigType(repository_type).into_response());
    };

    let Some(loaded_storage) = site.get_storage(request.storage) else {
        return Ok(Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body("Invalid Storage".into())
            .unwrap());
    };
    if !DBRepository::does_name_exist_for_storage(request.storage, &name, &site.database).await? {
        return Ok(Response::builder()
            .status(StatusCode::CONFLICT)
            .body("Name already in use".into())
            .unwrap());
    }

    let uuid = DBRepository::generate_uuid(&site.database).await?;
    let repository = repository_factory
        .create_new(name, uuid, configs, loaded_storage.clone())
        .await;
    let repository = match repository {
        Ok(repository) => repository,
        Err(err) => {
            error!("Failed to create repository: {}", err);
            return Ok(Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body("Failed to create repository".into())
                .unwrap());
        }
    };
    let db_repository = repository.insert(storage, site.as_ref()).await?;
    match repository_factory
        .load_repo(db_repository.clone(), loaded_storage, site.clone())
        .await
    {
        Ok(loaded) => {
            site.add_repository(db_repository.id, loaded);
        }
        Err(err) => {
            error!("Failed to load repository: {}", err);
            return Ok(Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body("Failed to load repository".into())
                .unwrap());
        }
    }
    Ok(Response::builder()
        .header(CONTENT_TYPE, "application/json")
        .status(StatusCode::CREATED)
        .body(serde_json::to_string(&db_repository).unwrap().into())
        .unwrap())
}

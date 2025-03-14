use ahash::HashMap;
use axum::{
    Json, Router,
    body::Body,
    debug_handler,
    extract::{Path, Query, State},
    response::{IntoResponse, Response},
    routing::{delete, get, post, put},
};
use http::{StatusCode, header::CONTENT_TYPE};
use nr_core::{
    database::entities::repository::{DBRepository, GenericDBRepositoryConfig},
    repository::Visibility,
    user::permissions::{HasPermissions, RepositoryActions},
};
use serde::Deserialize;
use serde_json::Value;
use tracing::{debug, error, info, instrument};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{
    app::{
        NitroRepo,
        authentication::{Authentication, AuthenticationError},
        responses::{InvalidRepositoryConfig, MissingPermission, RepositoryNotFound},
    },
    error::InternalError,
    repository::Repository,
    utils::ResponseBuilder,
};
pub fn management_routes() -> Router<NitroRepo> {
    Router::new()
        .route("/{repository_id}/configs", get(get_configs_for_repository))
        .route("/new/{repository_type}", post(new_repository))
        .route("/{repository_id}/config/{key}", put(update_config))
        .route("/{repository_id}/config/{key}", get(get_config))
        .route("/{repository_id}", delete(delete_repository))
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
    params(
        ("repository_type" = String, Path, description = "The Repository Type"),
    ),
    responses(
        (status = 200, description = "Create new Repository", body = DBRepository),
    )
)]
#[instrument]
pub async fn new_repository(
    State(site): State<NitroRepo>,
    auth: Authentication,
    Path(repository_type): Path<String>,
    Json(request): Json<NewRepositoryRequest>,
) -> Result<Response, InternalError> {
    if !auth.is_admin_or_system_manager() {
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
    if DBRepository::does_name_exist_for_storage(request.storage, &name, &site.database).await? {
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

#[utoipa::path(
    get,
    path = "/{repository_id}/configs",
    params(
        ("repository_id" = Uuid, Path,description = "The Repository ID"),
    ),
    responses(
        (status = 200, description = "List Configs for Repository", body = [String]),
    )
)]
#[instrument]
pub async fn get_configs_for_repository(
    State(site): State<NitroRepo>,
    auth: Authentication,
    Path(repository): Path<Uuid>,
) -> Result<Response, InternalError> {
    if !auth
        .has_action(RepositoryActions::Edit, repository, &site.database)
        .await?
    {
        return Ok(MissingPermission::EditRepository(repository).into_response());
    }
    let Some(repository) = site.get_repository(repository) else {
        return Ok(RepositoryNotFound::Uuid(repository).into_response());
    };

    let repository = repository.config_types();
    info!("Repository Configs: {:?}", repository);
    Ok(ResponseBuilder::ok().json(&repository))
}
#[derive(Deserialize, Default, Debug)]
#[serde(default)]
pub struct GetConfigParams {
    default: bool,
}
#[utoipa::path(
    get,
    path = "/{repository_id}/config/{config_key}",
    params(
        ("repository_id" = Uuid, Path, description = "The Repository ID"),
        ("config_key" = String, Path, description = "The Config Key"),
    ),
    responses(
        (status = 200, description = "Config for the repository"),
    )
)]
#[debug_handler]
#[instrument]
pub async fn get_config(
    State(site): State<NitroRepo>,
    auth: Option<Authentication>,
    Query(params): Query<GetConfigParams>,
    Path((repository, config)): Path<(Uuid, String)>,
) -> Result<Response, InternalError> {
    let repository_visibility = Visibility::Private;
    let Some(config_type) = site.get_repository_config_type(&config) else {
        return Ok(InvalidRepositoryConfig::InvalidConfigType(config).into_response());
    };
    let config =
        match GenericDBRepositoryConfig::get_config(repository, &config, site.as_ref()).await? {
            Some(config) => config.value.0,
            None => {
                if params.default {
                    debug!("Getting default config for config type: {}", config);
                    config_type.default()?
                } else {
                    return Ok(RepositoryNotFound::Uuid(repository).into_response());
                }
            }
        };
    let config = if auth
        .has_action(RepositoryActions::Edit, repository, &site.database)
        .await?
    {
        Some(config)
    } else {
        // User does not have permission to view the config. Sanitize it
        // If None is returned, the user does not have permission to view the config
        debug!("Sanitizing config for public view");
        match repository_visibility {
            Visibility::Hidden | Visibility::Public => {
                config_type.sanitize_for_public_view(config)?
            }
            _ => None,
        }
    };
    if let Some(config) = config {
        Ok(ResponseBuilder::ok().json(&config))
    } else {
        Ok(AuthenticationError::Forbidden.into_response())
    }
}
/// Updates a config for a repository
///
/// # Method Body
/// Should be a the message body for the type of config you are updating
#[utoipa::path(
    put,
    path = "/{repository_id}/config/{config_key}",
    params(
        ("repository_id" = Uuid,Path, description = "The Repository ID"),
        ("config_key" = String,Path, description = "The Config Key"),
    ),
    responses(
        (status = 204, description = "Updated a config for a repository"),
        (status = 404, description = "Repository not found"),
        (status = 400, description="Invalid Config value for the repository"),
    )
)]
#[instrument]
pub async fn update_config(
    State(site): State<NitroRepo>,
    auth: Authentication,
    Path((repository, config_key)): Path<(Uuid, String)>,
    Json(config): Json<serde_json::Value>,
) -> Result<Response, InternalError> {
    if !auth
        .has_action(RepositoryActions::Edit, repository, &site.database)
        .await?
    {
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
        let repository = repository.get_type();
        return Ok(InvalidRepositoryConfig::RepositoryTypeDoesntSupportConfig {
            repository_type: repository.to_owned(),
            config_key,
        }
        .into_response());
    }
    match GenericDBRepositoryConfig::get_config(repository.id(), &config_key, site.as_ref()).await?
    {
        Some(old) => {
            if let Err(error) = config_type.validate_change(old.value.0, config.clone()) {
                error!("Error validating config: {}", error);
                return Ok(
                    InvalidRepositoryConfig::InvalidConfig { config_key, error }.into_response()
                );
            }
        }
        None => {
            if let Err(error) = config_type.validate_config(config.clone()) {
                error!("Error validating config: {}", error);
                return Ok(
                    InvalidRepositoryConfig::InvalidConfig { config_key, error }.into_response()
                );
            }
        }
    };

    GenericDBRepositoryConfig::add_or_update(db_repository.id, config_key, config, site.as_ref())
        .await?;
    if let Err(err) = repository.reload().await {
        return Ok(Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(format!("Failed to reload repository: {}", err).into())
            .unwrap());
    }
    Ok(ResponseBuilder::no_content().empty())
}
#[utoipa::path(
    delete,
    path = "/{repository}",
    params(
        ("repository_id" = Uuid, description = "The Repository ID"),
    ),
    responses(
        (status = 204, description = "Repository Deleted"),
    )
)]
pub async fn delete_repository(
    State(site): State<NitroRepo>,
    auth: Authentication,
    Path(repository): Path<Uuid>,
) -> Result<Response, InternalError> {
    if !auth.is_admin_or_system_manager() {
        return Ok(MissingPermission::RepositoryManager.into_response());
    }
    let Some(db_repository) = DBRepository::get_by_id(repository, site.as_ref()).await? else {
        return Ok(RepositoryNotFound::Uuid(repository).into_response());
    };
    info!("Deleting Repository: {}", db_repository.name);
    DBRepository::delete_by_id(repository, site.as_ref()).await?;

    site.remove_repository(repository);
    // TODO: Delete all files for the repository
    Ok(Response::builder()
        .status(StatusCode::NO_CONTENT)
        .body(Body::empty())
        .unwrap())
}

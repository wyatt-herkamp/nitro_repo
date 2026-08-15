use ahash::HashMap;
use axum::{
    Json, Router,
    body::Body,
    debug_handler,
    extract::{Path, Query, State},
    response::{IntoResponse, Response},
    routing::{delete, get, post, put},
};
use http::StatusCode;
use nr_core::{
    database::entities::repository::{DBRepository, GenericDBRepositoryConfig},
    repository::Visibility,
    user::{
        permissions::{HasPermissions, RepositoryActions},
        scopes::NRScope,
    },
};
use nr_repository::RepositoryNotFound;
use nr_web_core::{
    authentication::{Authentication, AuthenticationError},
    error::InternalError,
    responses::{InvalidRepositoryConfig, MissingPermission},
    utils::{ResponseBuilder, conflict::ConflictResponse},
};
use serde::Deserialize;
use serde_json::Value;
use tracing::{debug, error, info, instrument};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::app::{NitroRepo, api::require_scope};
pub fn management_routes() -> Router<NitroRepo> {
    Router::new()
        .route("/{repository_id}/configs", get(get_configs_for_repository))
        .route("/new/{repository_type}", post(new_repository))
        .route("/{repository_id}/config/{key}", put(update_config))
        .route("/{repository_id}/config/{key}", get(get_config))
        .route("/{repository_id}", delete(delete_repository))
        .route("/{repository_id}", put(update_repository))
        .route("/{repository_id}/all-configs", get(get_all_configs))
}

/// Every config a repository has, in one request.
///
/// #506 calls the config API verbose, and it is: rendering a repository's settings meant one
/// request to list the config keys and then one more per key, each round trip re-doing the same
/// permission check. This returns them together, keyed by config type, with the same sanitization
/// rule the single-key route applies.
#[utoipa::path(
    get,
    path = "/{repository_id}/all-configs",
    params(("repository_id" = Uuid, Path, description = "The Repository ID")),
    responses(
        (status = 200, description = "Every config for the repository, keyed by type"),
        (status = 404, description = "Repository not found"),
    )
)]
#[instrument]
pub async fn get_all_configs(
    State(site): State<NitroRepo>,
    auth: Option<Authentication>,
    Path(repository): Path<Uuid>,
) -> Result<Response, InternalError> {
    let Some(db_repository) = DBRepository::get_by_id(repository, site.as_ref()).await? else {
        return Ok(RepositoryNotFound::Uuid(repository).into_response());
    };
    let Some(loaded) = site.get_repository(repository) else {
        return Ok(RepositoryNotFound::Uuid(repository).into_response());
    };
    let can_edit = auth
        .has_action(RepositoryActions::Edit, repository, &site.database)
        .await?;

    let mut configs: HashMap<String, Value> = HashMap::default();
    for key in loaded.config_types() {
        let Some(config_type) = site.get_repository_config_type(key) else {
            continue;
        };
        let stored =
            match GenericDBRepositoryConfig::get_config(repository, key, site.as_ref()).await? {
                Some(config) => config.value.0,
                // A config that has never been written still has a shape worth showing, and the
                // single-key route already serves the default on request.
                None => config_type.default()?,
            };
        let visible = if can_edit {
            Some(stored)
        } else {
            match db_repository.visibility {
                Visibility::Hidden | Visibility::Public => {
                    config_type.sanitize_for_public_view(stored)?
                }
                Visibility::Private => None,
            }
        };
        if let Some(value) = visible {
            configs.insert(key.to_owned(), value);
        }
    }
    Ok(ResponseBuilder::ok().json(&configs))
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
    if let Some(denied) = require_scope(&auth, NRScope::CreateRepository, &site).await? {
        return Ok(denied);
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
        return Ok(ConflictResponse::from("name").into_response());
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
        .load_repo(db_repository.clone(), loaded_storage, site.context())
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
    Ok(ResponseBuilder::created().json(&db_repository))
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
    // Read from the database. This was `let repository_visibility = Visibility::Private;` —
    // unconditional — so the `Hidden | Public` branch below was dead, `sanitize_for_public_view`
    // never ran, and every non-editor got a 403 on every config read regardless of how public the
    // repository was.
    let Some(db_repository) = DBRepository::get_by_id(repository, site.as_ref()).await? else {
        return Ok(RepositoryNotFound::Uuid(repository).into_response());
    };
    let repository_visibility = db_repository.visibility;
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
    if let Some(denied) = require_scope(&auth, NRScope::EditRepository, &site).await? {
        return Ok(denied);
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
/// What may be changed about an existing repository.
///
/// Omitted fields are left alone, so a rename does not have to restate the visibility.
#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateRepositoryRequest {
    pub name: Option<String>,
    pub visibility: Option<Visibility>,
    pub active: Option<bool>,
}

/// Renames a repository, or changes its visibility.
///
/// There was no update path — a repository's name was fixed at creation. Renaming is not
/// cosmetic: the name is in every artifact URL, so anything already pointing at the old name
/// stops resolving. That is the caller's decision to make, but it is worth them knowing.
#[utoipa::path(
    put,
    path = "/{repository_id}",
    request_body = UpdateRepositoryRequest,
    params(("repository_id" = Uuid, Path, description = "The Repository ID")),
    responses(
        (status = 200, description = "The updated repository", body = DBRepository),
        (status = 400, description = "The name is not valid"),
        (status = 404, description = "Repository not found"),
        (status = 409, description = "That name is already used in this storage"),
    )
)]
#[instrument]
pub async fn update_repository(
    State(site): State<NitroRepo>,
    auth: Authentication,
    Path(repository): Path<Uuid>,
    Json(request): Json<UpdateRepositoryRequest>,
) -> Result<Response, InternalError> {
    if !auth
        .has_action(RepositoryActions::Edit, repository, &site.database)
        .await?
    {
        return Ok(MissingPermission::EditRepository(repository).into_response());
    }
    if let Some(denied) = require_scope(&auth, NRScope::EditRepository, &site).await? {
        return Ok(denied);
    }
    let Some(existing) = DBRepository::get_by_id(repository, site.as_ref()).await? else {
        return Ok(RepositoryNotFound::Uuid(repository).into_response());
    };

    let name = match request.name {
        Some(name) if name != existing.name.as_ref() => {
            let name = match nr_core::repository::RepositoryName::new(name) {
                Ok(name) => name,
                Err(err) => return Ok(ResponseBuilder::bad_request().body(err.to_string())),
            };
            // Checked before writing so this reports a conflict rather than a constraint
            // violation; the unique index is still what makes it correct under a race.
            if DBRepository::does_name_exist_for_storage(existing.storage_id, &name, &site.database)
                .await?
            {
                return Ok(ConflictResponse::from("name").into_response());
            }
            Some(name)
        }
        _ => None,
    };

    DBRepository::update_details(
        repository,
        name.as_ref(),
        request.visibility,
        request.active,
        site.as_ref(),
    )
    .await?;

    // The loaded repository caches its name and visibility, so it has to be told. Without this a
    // rename takes effect in the database and the running instance keeps serving the old name.
    if let Some(loaded) = site.get_repository(repository)
        && let Err(error) = loaded.reload().await
    {
        error!(?error, "Renamed the repository but failed to reload it");
    }
    site.forget_repository_names(repository);

    let updated = DBRepository::get_by_id(repository, site.as_ref()).await?;
    Ok(ResponseBuilder::ok().json(&updated))
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
    if let Some(denied) = require_scope(&auth, NRScope::DeleteRepository, &site).await? {
        return Ok(denied);
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

use axum::{
    extract::{Path, State},
    response::{IntoResponse, Response},
    Json,
};
use http::StatusCode;
use nr_core::{
    database::repository::{DBRepository, DBRepositoryWithStorageName, GenericDBRepositoryConfig},
    user::permissions::HasPermissions,
};
use tracing::{error, instrument};
use utoipa::OpenApi;
use uuid::Uuid;

use crate::{
    app::{authentication::Authentication, NitroRepo},
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
        config::config_default
    ),
    components(schemas(DBRepository, DBRepositoryWithStorageName, RepositoryTypeDescription))
)]
pub struct RepositoryAPI;
pub fn repository_routes() -> axum::Router<NitroRepo> {
    axum::Router::new()
        .route("/list", axum::routing::get(list_repositories))
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
    //TODO: Permissions
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
) -> Result<StatusCode, InternalError> {
    if !auth.can_edit_repository(repository) {
        return Ok(StatusCode::FORBIDDEN);
    }
    let Some(config_type) = site.get_repository_config_type(&config_key) else {
        return Ok(StatusCode::BAD_REQUEST);
    };
    let Some(db_repository) = DBRepository::get_by_id(repository, site.as_ref()).await? else {
        return Ok(StatusCode::NOT_FOUND);
    };
    let Some(repository) = site.get_repository(db_repository.id) else {
        return Ok(StatusCode::INTERNAL_SERVER_ERROR);
    };
    if !repository.config_types().contains(&config_key) {
        return Ok(StatusCode::BAD_REQUEST);
    }
    if let Err(error) = config_type.validate_config(config.clone()) {
        error!("Error validating config: {}", error);
        return Ok(StatusCode::BAD_REQUEST);
    }
    GenericDBRepositoryConfig::add_or_update(db_repository.id, config_key, config, site.as_ref())
        .await?;
    //TODO: Update the instance of the repository with the new config
    Ok(StatusCode::OK)
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

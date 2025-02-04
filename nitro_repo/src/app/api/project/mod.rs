use axum::{
    extract::{Path, State},
    response::Response,
    routing::get,
};
use nr_core::{
    database::entities::project::{
        DBProject, ProjectDBType, utils::does_project_id_exist, versions::DBProjectVersion,
    },
    repository::project::ProjectResolution,
};
use tracing::instrument;
use utoipa::OpenApi;
use uuid::Uuid;

use crate::{
    app::{NitroRepo, authentication::Authentication},
    error::InternalError,
    utils::response_builder::ResponseBuilder,
};

#[derive(OpenApi)]
#[openapi(
    paths(get_project, get_project_versions, get_project_by_key),
    components(schemas(DBProject, ProjectResolution, DBProjectVersion))
)]
pub struct ProjectRoutes;
pub fn project_routes() -> axum::Router<NitroRepo> {
    axum::Router::new()
        .route("/{project_id}", get(get_project))
        .route("/{project_id}/versions", get(get_project_versions))
        .route(
            "/by-key/{repository_id}/{project_key}",
            get(get_project_by_key),
        )
}

#[utoipa::path(
    get,
    path = "/{project_id}",
    summary = "Get Project by its ID",
    params(
        ("project_id"  = Uuid, description = "The project ID")
    ),
    responses(
        (status = 200, description = "File listing", body = DBProject),
        (status = 404, description = "Project not found"),
        (status = 403, description = "Missing permission"),
    ),
)]
#[instrument]
pub async fn get_project(
    Path(project_id): Path<Uuid>,
    State(site): State<NitroRepo>,
    auth: Option<Authentication>,
) -> Result<Response, InternalError> {
    let Some(project) = DBProject::get_by_id(project_id, site.as_ref()).await? else {
        return Ok(ResponseBuilder::not_found().empty());
    };

    Ok(ResponseBuilder::ok().json(&project))
}

#[utoipa::path(
    get,
    path = "/{project_id}/versions",
    summary = "Get Project Versions",
    params(
        ("project_id" = Uuid, description = "The project ID")
    ),
    responses(
        (status = 200, description = "File listing", body = Vec<DBProjectVersion>),
        (status = 404, description = "Project not found"),
        (status = 403, description = "Missing permission"),
    ),
)]
#[instrument]
pub async fn get_project_versions(
    Path(project_id): Path<Uuid>,
    State(site): State<NitroRepo>,
    auth: Option<Authentication>,
) -> Result<Response, InternalError> {
    let versions = DBProjectVersion::get_all_versions(project_id, site.as_ref()).await?;

    if versions.is_empty() && !does_project_id_exist(project_id, site.as_ref()).await? {
        return Ok(ResponseBuilder::not_found().empty());
    }
    Ok(ResponseBuilder::ok().json(&versions))
}

#[utoipa::path(
    get,
    path = "/by-key/{repository_id}/{project_key}",
    summary = "Get Project by Key",
    params(
        ("repository_id" = Uuid, description = "The repository ID"),
        ("project_key" = String, description = "The project Key")
    ),
    responses(
        (status = 200, description = "File listing", body = DBProject),
        (status = 404, description = "Project not found"),
        (status = 403, description = "Missing permission"),
    ),
)]
#[instrument]
pub async fn get_project_by_key(
    Path((repository_id, project_key)): Path<(Uuid, String)>,
    State(site): State<NitroRepo>,
    auth: Option<Authentication>,
) -> Result<Response, InternalError> {
    let Some(project) =
        DBProject::find_by_project_key(&project_key, repository_id, site.as_ref()).await?
    else {
        return Ok(ResponseBuilder::not_found().empty());
    };

    Ok(ResponseBuilder::ok().json(&project))
}

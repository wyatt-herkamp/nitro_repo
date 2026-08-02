use axum::{
    extract::{Path, State},
    response::{IntoResponse, Response},
    routing::get,
};
use nr_core::{
    database::entities::project::{
        DBProject, ProjectDBType, ProjectResponse,
        versions::{DBProjectVersion, history::VersionHistoryItem},
    },
    repository::{Visibility, project::ProjectResolution},
    user::permissions::{HasPermissions, RepositoryActions},
};
use tracing::instrument;
use utoipa::OpenApi;
use uuid::Uuid;

use crate::{
    app::{NitroRepo, authentication::Authentication, responses::MissingPermission},
    error::InternalError,
    utils::ResponseBuilder,
};

/// Refuses a read of a project belonging to a repository the caller cannot see.
///
/// All three routes below took an `auth` argument and never looked at it, so every project and
/// every version list in a private repository was readable by anyone who could guess — or
/// enumerate — an id.
async fn check_can_read(
    auth: &Option<Authentication>,
    repository_id: Uuid,
    site: &NitroRepo,
) -> Result<Option<Response>, InternalError> {
    let Some(repository) = nr_core::database::entities::repository::DBRepository::get_by_id(
        repository_id,
        site.as_ref(),
    )
    .await?
    else {
        return Ok(Some(ResponseBuilder::not_found().empty()));
    };
    if matches!(repository.visibility, Visibility::Public) {
        return Ok(None);
    }
    if auth
        .has_action(RepositoryActions::Read, repository_id, site.as_ref())
        .await?
    {
        return Ok(None);
    }
    Ok(Some(
        MissingPermission::ReadRepository(repository_id).into_response(),
    ))
}

#[derive(OpenApi)]
#[openapi(
    paths(
        get_project,
        get_project_versions,
        get_project_by_key,
        get_project_version
    ),
    components(schemas(
        DBProject,
        ProjectResponse,
        ProjectResolution,
        DBProjectVersion,
        VersionHistoryItem
    ))
)]
pub struct ProjectRoutes;
pub fn project_routes() -> axum::Router<NitroRepo> {
    axum::Router::new()
        .route("/{project_id}", get(get_project))
        .route("/{project_id}/versions", get(get_project_versions))
        .route("/version/{version_id}", get(get_project_version))
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
        (status = 200, description = "The project", body = ProjectResponse),
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
    let Some(project) = DBProject::find_by_id(project_id, site.as_ref()).await? else {
        return Ok(ResponseBuilder::not_found().empty());
    };
    if let Some(denied) = check_can_read(&auth, project.repository_id, &site).await? {
        return Ok(denied);
    }
    let project = ProjectResponse::from_project(project, site.as_ref()).await?;

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
        (status = 200, description = "File listing", body = Vec<VersionHistoryItem>),
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
    let Some(project) = DBProject::find_by_id(project_id, site.as_ref()).await? else {
        return Ok(ResponseBuilder::not_found().empty());
    };
    if let Some(denied) = check_can_read(&auth, project.repository_id, &site).await? {
        return Ok(denied);
    }
    let versions = VersionHistoryItem::find_by_project_id(project_id, site.as_ref()).await?;

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
        (status = 200, description = "The project", body = ProjectResponse),
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
    if let Some(denied) = check_can_read(&auth, repository_id, &site).await? {
        return Ok(denied);
    }
    let Some(project) =
        DBProject::find_by_project_key(&project_key, repository_id, site.as_ref()).await?
    else {
        return Ok(ResponseBuilder::not_found().empty());
    };
    let project = ProjectResponse::from_project(project, site.as_ref()).await?;

    Ok(ResponseBuilder::ok().json(&project))
}

/// A single version, by id.
///
/// Browsing lands inside a version directory and resolves a `version_id`, but there was no way to
/// turn that id into a version string — so the install snippets shown next to a version's files
/// always fell back to "latest" instead of naming the version being looked at.
#[utoipa::path(
    get,
    path = "/version/{version_id}",
    summary = "Get a Project Version by its ID",
    params(
        ("version_id" = Uuid, description = "The version ID")
    ),
    responses(
        (status = 200, description = "The version", body = DBProjectVersion),
        (status = 404, description = "Version not found"),
        (status = 403, description = "Missing permission"),
    ),
)]
#[instrument]
pub async fn get_project_version(
    Path(version_id): Path<Uuid>,
    State(site): State<NitroRepo>,
    auth: Option<Authentication>,
) -> Result<Response, InternalError> {
    let Some(version) = DBProjectVersion::find_by_id(version_id, site.as_ref()).await? else {
        return Ok(ResponseBuilder::not_found().empty());
    };
    let Some(project) = DBProject::find_by_id(version.project_id, site.as_ref()).await? else {
        return Ok(ResponseBuilder::not_found().empty());
    };
    if let Some(denied) = check_can_read(&auth, project.repository_id, &site).await? {
        return Ok(denied);
    }

    Ok(ResponseBuilder::ok().json(&version))
}

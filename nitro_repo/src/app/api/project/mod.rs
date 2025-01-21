use axum::{
    extract::{Path, State},
    response::Response,
    routing::get,
};
use nr_core::{
    database::project::{DBProject, ProjectDBType},
    repository::project::ProjectResolution,
};
use tracing::instrument;
use utoipa::OpenApi;
use uuid::Uuid;

use crate::{
    app::{authentication::Authentication, NitroRepo},
    error::InternalError,
    utils::response_builder::ResponseBuilder,
};

#[derive(OpenApi)]
#[openapi(paths(get_project,), components(schemas(DBProject, ProjectResolution)))]
pub struct ProjectRoutes;
pub fn project_routes() -> axum::Router<NitroRepo> {
    axum::Router::new().route("/{project_id}", get(get_project))
}

#[utoipa::path(
    get,
    path = "/{project_id}",
    params(
        ("project_id", description = "The project ID")
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

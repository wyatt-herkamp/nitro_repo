use axum::{extract::State, response::Response};
use tracing::instrument;

use crate::{app::NitroRepo, repository::RepositoryTypeDescription, utils::ResponseBuilder};

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
    let types = site.inner.repository_types.descriptions();
    ResponseBuilder::ok().json(&types)
}

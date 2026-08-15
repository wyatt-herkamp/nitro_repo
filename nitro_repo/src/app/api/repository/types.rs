use axum::{extract::State, response::Response};
use nr_repository::RepositoryTypeDescription;
use nr_web_core::utils::ResponseBuilder;
use tracing::instrument;

use crate::app::NitroRepo;

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

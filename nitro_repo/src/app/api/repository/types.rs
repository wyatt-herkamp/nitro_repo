use axum::response::Response;
use tracing::instrument;

use crate::{app::REPOSITORY_TYPES, repository::RepositoryTypeDescription, utils::ResponseBuilder};

#[utoipa::path(
    get,
    path = "/types",
    responses(
        (status = 200, description = "Repository Types", body = [RepositoryTypeDescription]),
    )
)]
#[instrument]
pub async fn repository_types() -> Response {
    // TODO: Add Client side caching
    let types: Vec<_> = REPOSITORY_TYPES
        .iter()
        .map(|v| v.get_description())
        .collect();
    ResponseBuilder::ok().json(&types)
}

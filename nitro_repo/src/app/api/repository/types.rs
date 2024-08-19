use axum::response::Response;
use http::{header::CONTENT_TYPE, StatusCode};
use tracing::instrument;

use crate::app::REPOSITORY_TYPES;

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

    Response::builder()
        .status(StatusCode::OK)
        .header(CONTENT_TYPE, "application/json")
        .body(serde_json::to_string(&types).unwrap().into())
        .unwrap()
}

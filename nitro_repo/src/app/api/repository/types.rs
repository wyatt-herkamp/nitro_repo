use axum::response::{IntoResponse, Response};
use http::StatusCode;
use tracing::instrument;

use crate::app::{responses::ResponseBuilderExt, REPOSITORY_TYPES};

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
        .json_body(&types)
        .into_response()
}

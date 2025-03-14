use axum::response::IntoResponse;
use http::header::CONTENT_TYPE;
use thiserror::Error;

use crate::utils::{ErrorReason, other::PLAIN_TEXT_MEDIA_TYPE};

#[derive(Debug, Error)]
pub enum ResponseBuildError {
    #[error("Failed to serialize data for response: {0}")]
    SerdeError(#[from] serde_json::Error),
    #[error("Failed to build response: {0}")]
    HttpError(#[from] http::Error),
    #[error("Invalid Header Response Value: {0}")]
    HeaderValueError(#[from] http::header::InvalidHeaderValue),
}
impl IntoResponse for ResponseBuildError {
    fn into_response(self) -> axum::response::Response {
        let message = self.to_string();
        http::Response::builder()
            .status(http::StatusCode::INTERNAL_SERVER_ERROR)
            .header(CONTENT_TYPE, PLAIN_TEXT_MEDIA_TYPE)
            .extension(ErrorReason::from(self.to_string()))
            .body(axum::body::Body::from(message))
            .unwrap()
    }
}

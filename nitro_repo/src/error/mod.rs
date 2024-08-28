mod bad_requests;
mod internal_error;
use std::error::Error;

use axum::response::IntoResponse;
pub use bad_requests::*;
use derive_more::derive::From;
pub use internal_error::*;
#[derive(Debug, thiserror::Error)]
#[error("Illegal State: {0}")]
pub struct IllegalStateError(pub &'static str);
impl IntoResponse for IllegalStateError {
    fn into_response(self) -> axum::response::Response {
        axum::response::Response::builder()
            .status(http::StatusCode::INTERNAL_SERVER_ERROR)
            .body(axum::body::Body::from(self.to_string()))
            .unwrap()
    }
}
#[derive(Debug, From)]
pub struct SQLXError(pub sqlx::Error);
impl IntoResponse for SQLXError {
    fn into_response(self) -> axum::response::Response {
        axum::response::Response::builder()
            .status(http::StatusCode::INTERNAL_SERVER_ERROR)
            .body(
                format!(
                    "Internal Service Error. Please Contact the System Admin. Error: {}",
                    self.0
                )
                .into(),
            )
            .unwrap()
    }
}

pub trait IntoErrorResponse: Error + Send + Sync {
    fn into_response_boxed(self: Box<Self>) -> axum::response::Response;
}
impl IntoErrorResponse for sqlx::Error {
    fn into_response_boxed(self: Box<Self>) -> axum::response::Response {
        axum::response::Response::builder()
            .status(http::StatusCode::INTERNAL_SERVER_ERROR)
            .body(
                format!(
                    "Internal Service Error. Please Contact the System Admin. Error: {}",
                    self
                )
                .into(),
            )
            .unwrap()
    }
}

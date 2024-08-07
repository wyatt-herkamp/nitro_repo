mod bad_requests;
mod internal_error;
use axum::response::IntoResponse;
pub use bad_requests::*;
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

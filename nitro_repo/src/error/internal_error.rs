use axum::response::{IntoResponse, Response};
use http::StatusCode;
use thiserror::Error;
use tracing::error;

use crate::app::authentication::session::SessionError;

/// Errors that happen internally to the system.
/// Not as a direct result of a Request
#[derive(Error, Debug)]
pub enum InternalError {
    #[error("Json Parsing error {0}")]
    JSONError(#[from] serde_json::Error),
    #[error("Internal IO error {0}")]
    IOError(#[from] std::io::Error),
    #[error("Database error {0}")]
    DBError(#[from] sqlx::Error),
    #[error("Password Hash Error: {0}")]
    PasswordHashError(#[from] argon2::password_hash::Error),
    #[error("Argon2 Error: {0}")]
    Argon2Error(#[from] argon2::Error),
    #[error("Session Error {0}")]
    SessionError(#[from] SessionError),
    #[error("Storage Error {0}")]
    StorageError(#[from] nr_storage::StorageError),
}

impl IntoResponse for InternalError {
    fn into_response(self) -> Response {
        match self {
            Self::SessionError(err) => err.into_response(),
            other => {
                error!("{}", other);
                let message = format!(
                    "Internal Service Error. Please Contact the System Admin. Error: {}",
                    other
                );
                Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .body(message.into())
                    .unwrap()
            }
        }
    }
}

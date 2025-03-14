use std::fmt::Display;

use axum::{body::Body, response::IntoResponse, response::Response};
use http::StatusCode;
use thiserror::Error;

use crate::{
    app::authentication::AuthenticationError,
    utils::{IntoErrorResponse, bad_request::BadRequestErrors},
};

#[derive(Debug, Error)]
pub enum RepositoryHandlerError {
    #[error("Database Error: {0}")]
    SQLXError(#[from] sqlx::Error),
    #[error("Storage Error: {0}")]
    StorageError(#[from] nr_storage::StorageError),
    #[error("Unexpected Missing Body")]
    MissingBody,
    #[error("Invalid JSON: {0}")]
    InvalidJson(#[from] serde_json::Error),
    #[error("IO Error: {0}")]
    IOError(#[from] std::io::Error),
    #[error("Authentication Error: {0}")]
    AuthenticationError(#[from] AuthenticationError),
    #[error("{0}")]
    Other(Box<dyn IntoErrorResponse>),
}

impl From<BadRequestErrors> for RepositoryHandlerError {
    fn from(error: BadRequestErrors) -> Self {
        RepositoryHandlerError::Other(Box::new(error))
    }
}

impl IntoResponse for RepositoryHandlerError {
    fn into_response(self) -> Response {
        match self {
            RepositoryHandlerError::StorageError(error) => Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(Body::from(format!(
                    "Error from Internal Storage System. Please contact your admin \n {}",
                    error
                )))
                .unwrap(),
            RepositoryHandlerError::Other(error) => error.into_response_boxed(),
            other => Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(Body::from(format!(
                    "Internal Service Error  Please contact your admin \n {}",
                    other
                )))
                .unwrap(),
        }
    }
}
impl IntoErrorResponse for RepositoryHandlerError {
    fn into_response_boxed(self: Box<Self>) -> Response {
        self.into_response()
    }
}

/// A DynRepositoryHandlerError is a boxed version of a IntoErrorResponse
///
/// impl From<ErrorType> for DynRepositoryHandlerError  is required because we can't impl IntoBoxedResponse for DynRepositoryHandlerError or
///  it will create conflicting implementations
#[derive(Debug)]
pub struct DynRepositoryHandlerError(pub Box<dyn IntoErrorResponse>);
impl Display for DynRepositoryHandlerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl std::error::Error for DynRepositoryHandlerError {}
impl IntoResponse for DynRepositoryHandlerError {
    fn into_response(self) -> axum::response::Response {
        self.0.into_response_boxed()
    }
}

impl IntoErrorResponse for DynRepositoryHandlerError {
    fn into_response_boxed(self: Box<Self>) -> axum::response::Response {
        self.into_response()
    }
}

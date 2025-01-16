mod bad_requests;
use std::{error::Error, fmt::Display, io};

use axum::{body::Body, response::IntoResponse};
pub use bad_requests::*;
use http::header::CONTENT_TYPE;
use nr_core::repository::config::RepositoryConfigError;
//pub use internal_error::*;
use nr_storage::StorageError;
use thiserror::Error;

use crate::utils::TEXT_MEDIA_TYPE;

/// Allows creating a response from an error
pub trait IntoErrorResponse: Error + Send + Sync {
    /// Converts the error into a response
    ///
    /// It must be of type of Box<Self> to allow for dynamic dispatch
    fn into_response_boxed(self: Box<Self>) -> axum::response::Response;
}
macro_rules! impl_into_error_response_for_axum_into_response {
    ($t:ty) => {
        impl IntoErrorResponse for $t {
            fn into_response_boxed(self: Box<Self>) -> axum::response::Response {
                <Self as axum::response::IntoResponse>::into_response(*self)
            }
        }
    };
}

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
impl_into_error_response_for_axum_into_response!(IllegalStateError);

fn internal_error_message(err: impl Error) -> Body {
    format!(
        "Internal Service Error. Please Contact the System Admin. Error: {}",
        err
    )
    .into()
}
impl IntoErrorResponse for reqwest::Error {
    fn into_response_boxed(self: Box<Self>) -> axum::response::Response {
        axum::response::Response::builder()
            .status(http::StatusCode::INTERNAL_SERVER_ERROR)
            .body(internal_error_message(self))
            .unwrap()
    }
}
impl IntoErrorResponse for io::Error {
    fn into_response_boxed(self: Box<Self>) -> axum::response::Response {
        axum::response::Response::builder()
            .status(http::StatusCode::INTERNAL_SERVER_ERROR)
            .body(internal_error_message(self))
            .unwrap()
    }
}
impl IntoErrorResponse for StorageError {
    fn into_response_boxed(self: Box<Self>) -> axum::response::Response {
        axum::response::Response::builder()
            .status(http::StatusCode::INTERNAL_SERVER_ERROR)
            .body(internal_error_message(self))
            .unwrap()
    }
}
impl IntoErrorResponse for sqlx::Error {
    fn into_response_boxed(self: Box<Self>) -> axum::response::Response {
        axum::response::Response::builder()
            .status(http::StatusCode::INTERNAL_SERVER_ERROR)
            .body(internal_error_message(self))
            .unwrap()
    }
}
impl IntoErrorResponse for serde_json::Error {
    fn into_response_boxed(self: Box<Self>) -> axum::response::Response {
        axum::response::Response::builder()
            .status(http::StatusCode::INTERNAL_SERVER_ERROR)
            .body(internal_error_message(self))
            .unwrap()
    }
}
impl IntoErrorResponse for http::Error {
    fn into_response_boxed(self: Box<Self>) -> axum::response::Response {
        axum::response::Response::builder()
            .status(http::StatusCode::INTERNAL_SERVER_ERROR)
            .body(internal_error_message(self))
            .unwrap()
    }
}
impl IntoErrorResponse for argon2::Error {
    fn into_response_boxed(self: Box<Self>) -> axum::response::Response {
        axum::response::Response::builder()
            .status(http::StatusCode::INTERNAL_SERVER_ERROR)
            .body(internal_error_message(self))
            .unwrap()
    }
}
impl IntoErrorResponse for argon2::password_hash::Error {
    fn into_response_boxed(self: Box<Self>) -> axum::response::Response {
        axum::response::Response::builder()
            .status(http::StatusCode::INTERNAL_SERVER_ERROR)
            .body(internal_error_message(self))
            .unwrap()
    }
}
impl IntoErrorResponse for RepositoryConfigError {
    fn into_response_boxed(self: Box<Self>) -> axum::response::Response {
        axum::response::Response::builder()
            .status(http::StatusCode::INTERNAL_SERVER_ERROR)
            .body(internal_error_message(self))
            .unwrap()
    }
}
#[derive(Debug)]
pub struct InternalError(pub Box<dyn IntoErrorResponse>);
impl Display for InternalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl Error for InternalError {}

impl IntoResponse for InternalError {
    fn into_response(self) -> axum::response::Response {
        self.0.into_response_boxed()
    }
}

impl<T: IntoErrorResponse + 'static> From<T> for InternalError {
    fn from(err: T) -> Self {
        InternalError(Box::new(err))
    }
}

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
            .header(CONTENT_TYPE, TEXT_MEDIA_TYPE)
            .body(axum::body::Body::from(message))
            .unwrap()
    }
}

impl IntoErrorResponse for ResponseBuildError {
    fn into_response_boxed(self: Box<Self>) -> axum::response::Response {
        self.into_response()
    }
}

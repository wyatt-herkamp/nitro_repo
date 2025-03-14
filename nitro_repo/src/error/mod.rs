use std::{borrow::Cow, error::Error, fmt::Display, io};

use axum::response::IntoResponse;
use nr_core::{database::DBError, repository::config::RepositoryConfigError};
//pub use internal_error::*;
use nr_storage::StorageError;
use thiserror::Error;

use crate::utils::{IntoErrorResponse, ResponseBuilder, api_error_response::APIErrorResponse};

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
fn internal_error_response(err: impl Error, section: &'static str) -> axum::response::Response {
    let api_error_response = APIErrorResponse {
        message: Cow::Borrowed("Internal Service Error. Please Contact the System Admin."),
        details: Some(section),
        error: Some(Box::new(err)),
    };

    ResponseBuilder::internal_server_error().json(&api_error_response)
}
macro_rules! default_internal_errors {
    (
        $(
            $section:literal -> $t:ty
        ),*
    ) => {
        $(
            impl IntoErrorResponse for $t {
                #[inline(always)]
                fn into_response_boxed(self: Box<Self>) -> axum::response::Response {
                    internal_error_response(*self, $section)
                }
            }
        )*
    };
}

default_internal_errors!(
    "network-request" -> reqwest::Error,
    "io" -> io::Error,
    "storage" -> StorageError,
    "database" -> sqlx::Error,
    "database" -> DBError,
    "json" -> serde_json::Error,
    "argon2" -> argon2::Error,
    "argon2" -> argon2::password_hash::Error,
    "repository-config" -> RepositoryConfigError,
    "internal-server" -> http::Error,
    "internal-server" -> axum::Error
);
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
#[error("Internal Error: {0}")]
pub struct OtherInternalError(pub Box<dyn Error + Send + Sync>);
impl OtherInternalError {
    pub fn new<E>(err: E) -> Self
    where
        E: Error + Send + Sync + 'static,
    {
        OtherInternalError(Box::new(err))
    }
}

impl IntoErrorResponse for OtherInternalError {
    fn into_response_boxed(self: Box<Self>) -> axum::response::Response {
        let response_json = APIErrorResponse {
            message: Cow::Borrowed("Internal Error"),
            details: Option::<()>::None,
            error: Some(self.0),
        };
        ResponseBuilder::internal_server_error().json(&response_json)
    }
}

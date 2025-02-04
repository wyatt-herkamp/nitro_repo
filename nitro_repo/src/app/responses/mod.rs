use std::fmt::Debug;

use axum::{
    body::Body,
    response::{IntoResponse, Response},
};
use derive_more::derive::From;
use http::{HeaderValue, StatusCode, header::CONTENT_TYPE};
use mime::Mime;
use nr_core::repository::config::RepositoryConfigError;
use nr_storage::StorageError;
use redb::Result;
use tracing::instrument;

use crate::error::InternalError;

use super::RepositoryStorageName;
#[derive(Debug, From)]
pub enum RepositoryNotFound {
    RepositoryAndNameLookup(RepositoryStorageName),
    Uuid(uuid::Uuid),
}
impl IntoResponse for RepositoryNotFound {
    fn into_response(self) -> axum::response::Response {
        match self {
            Self::RepositoryAndNameLookup(lookup) => Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Body::from(format!(
                    "Repository {}/{} not found",
                    lookup.storage_name, lookup.repository_name
                )))
                .unwrap(),
            Self::Uuid(uuid) => Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Body::from(format!("Repository not found: {:?}", uuid)))
                .unwrap(),
        }
    }
}

#[derive(Debug)]
pub enum MissingPermission {
    UserManager,
    RepositoryManager,
    EditRepository(uuid::Uuid),
    ReadRepository(uuid::Uuid),
    StorageManager,
}
impl IntoResponse for MissingPermission {
    #[inline(always)]
    #[instrument(name = "MissingPermission::into_response", skip(self))]
    fn into_response(self) -> axum::response::Response {
        match self {
            Self::UserManager => Response::builder()
                .status(StatusCode::FORBIDDEN)
                .body(Body::from("You are not a user manager or admin"))
                .unwrap(),
            Self::RepositoryManager => Response::builder()
                .status(StatusCode::FORBIDDEN)
                .body(Body::from("You are not a repository manager or admin"))
                .unwrap(),
            Self::EditRepository(id) => Response::builder()
                .status(StatusCode::FORBIDDEN)
                .body(Body::from(format!(
                    "You do not have permission to edit repository: {}",
                    id
                )))
                .unwrap(),
            Self::ReadRepository(id) => Response::builder()
                .status(StatusCode::FORBIDDEN)
                .body(Body::from(format!(
                    "You do not have permission to read repository: {}",
                    id
                )))
                .unwrap(),
            Self::StorageManager => Response::builder()
                .status(StatusCode::FORBIDDEN)
                .body(Body::from("You are not a storage manager or admin"))
                .unwrap(),
        }
    }
}
#[derive(Debug, From)]
pub struct InvalidStorageType(pub String);
impl IntoResponse for InvalidStorageType {
    fn into_response(self) -> Response {
        Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Body::from(format!("Invalid Storage Type: {}", self.0)))
            .unwrap()
    }
}
#[derive(Debug, From)]
pub struct InvalidStorageConfig(pub StorageError);

impl IntoResponse for InvalidStorageConfig {
    fn into_response(self) -> Response {
        Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Body::from(format!("Invalid Storage Config: {}", self.0)))
            .unwrap()
    }
}

#[derive(Debug, From)]
pub enum InvalidRepositoryConfig {
    InvalidConfigType(String),
    RepositoryTypeDoesntSupportConfig {
        repository_type: String,
        config_key: String,
    },
    InvalidConfig {
        config_key: String,
        error: RepositoryConfigError,
    },
}
impl IntoResponse for InvalidRepositoryConfig {
    fn into_response(self) -> Response {
        match self {
            Self::InvalidConfigType(t) => Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .body(Body::from(format!("Invalid Repository Config Type: {}", t)))
                .unwrap(),
            Self::RepositoryTypeDoesntSupportConfig {
                repository_type,
                config_key,
            } => Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .body(Body::from(format!(
                    "Repository Type {} does not support config key {}",
                    repository_type, config_key
                )))
                .unwrap(),
            Self::InvalidConfig { config_key, error } => Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .body(Body::from(format!(
                    "Invalid Config for key {}: {}",
                    config_key, error
                )))
                .unwrap(),
        }
    }
}
pub trait ResponseBuilderExt {
    /// Adds a `Content-Type: application/json` header and serializes the given body to JSON.
    /// The type must also implement `Debug` for logging purposes.
    /// ## Errors
    /// - If serialization fails
    /// - If building the response fails
    fn json_body<T: serde::Serialize + Debug>(self, body: &T) -> Result<Response, InternalError>;

    fn content_type(self, content_type: impl Into<HeaderValue>) -> Self;

    fn mime_type(self, mime: Mime) -> Self;
}
const JSON_MIME_TYPE: HeaderValue = HeaderValue::from_static("application/json");

impl ResponseBuilderExt for http::response::Builder {
    fn json_body<T: serde::Serialize + Debug>(self, body: &T) -> Result<Response, InternalError> {
        let body: Body = serde_json::to_string(body)
            .map_err(|error| {
                tracing::error!(
                    ?error,
                    ?body,
                    "Failed to serialize body for type {}",
                    std::any::type_name::<T>()
                );
                InternalError::from(error)
            })?
            .into();
        self.header(CONTENT_TYPE, JSON_MIME_TYPE)
            .body(body)
            .map_err(InternalError::from)
    }

    fn content_type(self, content_type: impl Into<HeaderValue>) -> Self {
        self.header(CONTENT_TYPE, content_type)
    }
    fn mime_type(self, mime: Mime) -> Self {
        self.header(CONTENT_TYPE, mime.to_string())
    }
}
pub fn no_content_response() -> Response {
    Response::builder()
        .status(StatusCode::NO_CONTENT)
        .body(Body::empty())
        .unwrap()
}
pub fn no_content_response_with_error<T: From<http::Error>>() -> Result<Response, T> {
    Response::builder()
        .status(StatusCode::NO_CONTENT)
        .body(Body::empty())
        .map_err(T::from)
}

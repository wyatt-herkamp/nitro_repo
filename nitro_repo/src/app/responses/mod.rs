use std::fmt::Debug;

use axum::{
    body::Body,
    response::{IntoResponse, Response},
};
use derive_more::derive::From;
use http::StatusCode;
use nr_core::repository::config::RepositoryConfigError;
use nr_storage::StorageError;
use tracing::instrument;

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

//! Response types the API returns from more than one place.
//!
//! `RepositoryNotFound` is not here — it lives with the lookup that produces it, in
//! `nr-repository`.

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

// Moved next to the lookup it reports on; re-exported so `app::responses::RepositoryNotFound`
// still resolves.

#[derive(Debug)]
pub enum MissingPermission {
    UserManager,
    RepositoryManager,
    EditRepository(uuid::Uuid),
    ReadRepository(uuid::Uuid),
    WriteRepository(uuid::Uuid),
    StorageManager,
    /// The user is allowed to do this, but the token they used is not.
    ///
    /// Kept distinct from the permission failures above because the fix is different: the caller
    /// needs a new token, not a new role, and saying "forbidden" without that detail sends people
    /// looking in the wrong place.
    Scope(nr_core::user::scopes::NRScope),
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
            Self::WriteRepository(id) => Response::builder()
                .status(StatusCode::FORBIDDEN)
                .body(Body::from(format!(
                    "You do not have permission to write to repository: {}",
                    id
                )))
                .unwrap(),
            Self::StorageManager => Response::builder()
                .status(StatusCode::FORBIDDEN)
                .body(Body::from("You are not a storage manager or admin"))
                .unwrap(),
            Self::Scope(scope) => Response::builder()
                .status(StatusCode::FORBIDDEN)
                .body(Body::from(format!(
                    "The token used does not carry the `{scope}` scope"
                )))
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

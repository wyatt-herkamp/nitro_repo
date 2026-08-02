//! Cargo Registry Implementation
//!
//! Implements the sparse (HTTP) registry protocol, which has been the default since Rust 1.68.
//! There is no git index: a sparse registry is a set of plain files, so the existing storage and
//! routing carry it without a git server in the way.
//!
//! Protocol references:
//! - Index: <https://doc.rust-lang.org/cargo/reference/registry-index.html>
//! - Web API: <https://doc.rust-lang.org/cargo/reference/registry-web-api.html>

use ahash::HashMap;
use futures::future::BoxFuture;
use hosted::CargoHostedRegistry;
use nr_core::{
    database::entities::repository::{DBRepository, DBRepositoryConfig},
    repository::config::{
        RepositoryConfigType, project::ProjectConfigType, repository_page::RepositoryPageType,
    },
};
use nr_macros::DynRepositoryHandler;
use nr_storage::DynStorage;

pub mod hosted;
pub mod types;
pub mod utils;

mod configs;
pub use configs::*;

pub use super::prelude::*;
use super::{DynRepository, NewRepository, RepositoryType, RepositoryTypeDescription};
use crate::{
    app::authentication::AuthenticationError,
    utils::{IntoErrorResponse, bad_request::BadRequestErrors},
};

#[derive(Debug, Clone, DynRepositoryHandler)]
#[repository_handler(error=CargoRegistryError)]
pub enum CargoRegistry {
    Hosted(hosted::CargoHostedRegistry),
}

#[derive(Debug, thiserror::Error)]
pub enum CargoRegistryError {
    #[error("{0}")]
    NotFound(String),
    #[error(
        "The publish body is not in the format cargo sends: the {0} frame is missing or its \
         declared length runs past the end of the body."
    )]
    MalformedPublishBody(&'static str),
    #[error("`{0}` is not a valid crate name")]
    InvalidCrateName(String),
    #[error("`{0}` is not a valid semver version")]
    InvalidVersion(String),
    /// crates.io refuses this and so does this registry — an immutable version is the assumption
    /// every lockfile and every `cksum` in the index is built on.
    #[error(
        "crate version `{name}@{version}` already exists. Publish a new version instead; a \
         published version cannot be replaced."
    )]
    VersionAlreadyExists { name: String, version: String },
    #[error("The stored index record for `{name}@{version}` will not parse")]
    CorruptIndexRecord { name: String, version: String },
    #[error("This registry has no `app_url` configured, so it cannot tell cargo where to look")]
    NoAppUrl,
    #[error("Invalid storage path: {0}")]
    InvalidPath(#[from] nr_core::storage::InvalidStoragePath),
    #[error("{0}")]
    Other(Box<dyn IntoErrorResponse>),
}

impl From<CargoRegistryError> for RepositoryHandlerError {
    fn from(err: CargoRegistryError) -> Self {
        RepositoryHandlerError::Other(Box::new(err))
    }
}

macro_rules! impl_from_error_for_other {
    ($t:ty) => {
        impl From<$t> for CargoRegistryError {
            fn from(e: $t) -> Self {
                CargoRegistryError::Other(Box::new(e))
            }
        }
    };
}
impl_from_error_for_other!(BadRequestErrors);
impl_from_error_for_other!(sqlx::Error);
impl_from_error_for_other!(nr_core::database::DBError);
impl_from_error_for_other!(serde_json::Error);
impl_from_error_for_other!(std::io::Error);
impl_from_error_for_other!(AuthenticationError);
impl_from_error_for_other!(RepositoryHandlerError);
impl_from_error_for_other!(nr_storage::StorageError);

impl IntoErrorResponse for CargoRegistryError {
    fn into_response_boxed(self: Box<Self>) -> Response {
        self.into_response()
    }
}

impl From<CargoRegistryError> for DynRepositoryHandlerError {
    fn from(err: CargoRegistryError) -> Self {
        DynRepositoryHandlerError(Box::new(err))
    }
}

/// The error shape cargo expects.
///
/// Cargo reads `errors[].detail` out of the body and prints it. A plain-text body reaches the user
/// as the HTTP status and nothing else, which is how "failed to publish" ends up with no reason
/// attached.
pub fn cargo_error(status: StatusCode, detail: &str) -> Response {
    let body = serde_json::json!({ "errors": [ { "detail": detail } ] }).to_string();
    Response::builder()
        .status(status)
        .header(http::header::CONTENT_TYPE, "application/json")
        .body(body.into())
        .unwrap()
}

impl IntoResponse for CargoRegistryError {
    fn into_response(self) -> Response {
        match self {
            CargoRegistryError::NotFound(ref message) => {
                cargo_error(StatusCode::NOT_FOUND, message)
            }
            CargoRegistryError::VersionAlreadyExists { .. } => {
                cargo_error(StatusCode::CONFLICT, &self.to_string())
            }
            CargoRegistryError::MalformedPublishBody(_)
            | CargoRegistryError::InvalidCrateName(_)
            | CargoRegistryError::InvalidVersion(_)
            | CargoRegistryError::InvalidPath(_) => {
                cargo_error(StatusCode::BAD_REQUEST, &self.to_string())
            }
            CargoRegistryError::CorruptIndexRecord { .. } | CargoRegistryError::NoAppUrl => {
                cargo_error(StatusCode::INTERNAL_SERVER_ERROR, &self.to_string())
            }
            CargoRegistryError::Other(other) => other.into_response_boxed(),
        }
    }
}

#[derive(Debug, Default)]
pub struct CargoRegistryType;

impl RepositoryType for CargoRegistryType {
    fn get_type(&self) -> &'static str {
        "cargo"
    }

    fn config_types(&self) -> Vec<&str> {
        vec![
            CargoRegistryConfigType::get_type_static(),
            RepositoryPageType::get_type_static(),
            ProjectConfigType::get_type_static(),
        ]
    }

    fn get_description(&self) -> RepositoryTypeDescription {
        RepositoryTypeDescription {
            type_name: "cargo",
            name: "Cargo",
            description: "A Cargo registry, speaking the sparse index protocol",
            documentation_url: Some("https://nitro-repo.kingtux.dev/repositories/cargo/"),
            is_stable: true,
            required_configs: vec![CargoRegistryConfigType::get_type_static()],
        }
    }

    fn create_new(
        &self,
        name: String,
        uuid: uuid::Uuid,
        configs: HashMap<String, serde_json::Value>,
        _storage: DynStorage,
    ) -> BoxFuture<'static, Result<NewRepository, RepositoryFactoryError>> {
        Box::pin(async move {
            let sub_type = configs
                .get(CargoRegistryConfigType::get_type_static())
                .ok_or(RepositoryFactoryError::MissingConfig(
                    CargoRegistryConfigType::get_type_static(),
                ))?
                .clone();
            if let Err(err) = serde_json::from_value::<CargoRegistryConfig>(sub_type) {
                return Err(RepositoryFactoryError::InvalidConfig(
                    CargoRegistryConfigType::get_type_static(),
                    err.to_string(),
                ));
            }
            Ok(NewRepository {
                name,
                uuid,
                repository_type: "cargo".to_string(),
                configs,
            })
        })
    }

    fn load_repo(
        &self,
        repo: DBRepository,
        storage: DynStorage,
        website: NitroRepo,
    ) -> BoxFuture<'static, Result<DynRepository, RepositoryFactoryError>> {
        Box::pin(async move {
            let Some(config) = DBRepositoryConfig::<CargoRegistryConfig>::get_config(
                repo.id,
                CargoRegistryConfigType::get_type_static(),
                &website.database,
            )
            .await?
            else {
                return Err(RepositoryFactoryError::MissingConfig(
                    CargoRegistryConfigType::get_type_static(),
                ));
            };
            match config.value.0 {
                CargoRegistryConfig::Hosted => {
                    let hosted = CargoHostedRegistry::load(website, storage, repo).await?;
                    Ok(CargoRegistry::Hosted(hosted).into())
                }
            }
        })
    }
}

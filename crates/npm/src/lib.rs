//! NPM Registry Implementation
//!
//! Documentation for NPM: <https://github.com/npm/registry/blob/main/docs/REGISTRY-API.md>
//!

use std::{borrow::Cow, sync::Arc};

use ahash::HashMap;
use base64::DecodeError;
use config::RepositoryConfigType;
use futures::future::BoxFuture;
use hosted::NPMHostedRegistry;
use nr_core::database::entities::repository::{DBRepository, DBRepositoryConfig};
use nr_macros::DynRepositoryHandler;
use nr_storage::DynStorage;
use tracing::debug;
use types::InvalidNPMPackageName;

pub mod api;
pub mod hosted;
pub mod integrity;
pub mod login;
pub mod search;
pub mod types;
pub mod utils;
use login::web_login::NpmWebLoginManager;
use nr_core::repository::config::{
    project::ProjectConfigType, repository_page::RepositoryPageType,
};
// The prelude is what the `DynRepositoryHandler` derive expects in scope at the derive site.
use nr_repository::prelude::*;
use nr_web_core::{
    authentication::AuthenticationError,
    utils::{IntoErrorResponse, bad_request::BadRequestErrors},
};
mod configs;
pub use configs::*;
use nr_repository::{
    DynRepository, NewRepository, RepositoryHandlerError, RepositoryType, RepositoryTypeDescription,
};

#[derive(Debug, Clone, DynRepositoryHandler)]
#[repository_handler(error=NPMRegistryError)]
pub enum NPMRegistry {
    Hosted(hosted::NPMHostedRegistry),
}

#[derive(Debug, thiserror::Error)]
pub enum NPMRegistryError {
    #[error(transparent)]
    InvalidName(#[from] InvalidNPMPackageName),
    #[error(
        "Invalid tarball. The tarballs location is invalid.
        This means you used `$BASE_URL/repositories/$STORAGE/$REPO` without a trailing slash.
        tarbar Route: {tarball_route} Error: {error}"
    )]
    InvalidTarball {
        tarball_route: String,
        error: Cow<'static, str>,
    },
    #[error(
        "Invalid GET request. The requested route is invalid to the NPM Registry. This could be a bug. AS the code is very sketchy"
    )]
    InvalidGetRequest,
    #[error("Invalid Package Attachment. Error: {0}")]
    InvalidPackageAttachment(DecodeError),
    #[error("Only one release or attachment can be uploaded at a time")]
    OnlyOneReleaseOrAttachmentAtATime,
    /// npm's own registry refuses this, and so does this one. Overwriting in place used to leave
    /// the stored tarball and the recorded metadata describing different things.
    #[error(
        "You cannot publish over the previously published versions: {version}. \
         Unpublish it first, or publish a new version."
    )]
    VersionAlreadyExists { version: String },
    #[error(transparent)]
    Integrity(#[from] integrity::IntegrityError),
    #[error("{message}")]
    NotFound { message: String },
    #[error("The `{0}` command is not supported by this registry")]
    UnsupportedCommand(String),
    #[error("{0}")]
    Other(Box<dyn IntoErrorResponse>),
}
impl From<NPMRegistryError> for RepositoryHandlerError {
    fn from(err: NPMRegistryError) -> Self {
        RepositoryHandlerError::Other(Box::new(err))
    }
}
macro_rules! impl_from_error_for_other {
    ($t:ty) => {
        impl From<$t> for NPMRegistryError {
            fn from(e: $t) -> Self {
                NPMRegistryError::Other(Box::new(e))
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

impl IntoErrorResponse for NPMRegistryError {
    fn into_response_boxed(self: Box<Self>) -> axum::response::Response {
        self.into_response()
    }
}

/// The error shape npm expects. The CLI prints `error` verbatim, so a plain-text body reaches the
/// user as an unhelpful "Unexpected token" instead of the message.
fn npm_error(status: StatusCode, message: &str) -> Response {
    let body = serde_json::json!({ "error": message }).to_string();
    Response::builder()
        .status(status)
        .header(http::header::CONTENT_TYPE, "application/json")
        .body(body.into())
        .unwrap()
}

impl IntoResponse for NPMRegistryError {
    fn into_response(self) -> Response {
        match self {
            NPMRegistryError::InvalidGetRequest => Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body("Invalid GET request".into())
                .unwrap(),
            // npm reads the `error` field out of the JSON body and shows it to the user, so these
            // are worth answering in the shape the client expects rather than as bare text.
            NPMRegistryError::NotFound { ref message } => npm_error(StatusCode::NOT_FOUND, message),
            NPMRegistryError::VersionAlreadyExists { .. } => {
                npm_error(StatusCode::CONFLICT, &self.to_string())
            }
            NPMRegistryError::Integrity(_) => npm_error(StatusCode::BAD_REQUEST, &self.to_string()),
            NPMRegistryError::UnsupportedCommand(_) => {
                npm_error(StatusCode::NOT_IMPLEMENTED, &self.to_string())
            }
            NPMRegistryError::Other(other) => other.into_response_boxed(),
            bad_request => {
                debug!("Bad Request: {:?}", bad_request);
                Response::builder()
                    .status(StatusCode::BAD_REQUEST)
                    .body(bad_request.to_string().into())
                    .unwrap()
            }
        }
    }
}
/// The value stored in `repositories.repository_type`.
///
/// A named constant rather than a literal at each of the three sites that produce it: the column
/// is a free `VARCHAR` with no constraint, so a typo here is a repository nobody can load, and the
/// failure lands at startup naming nothing useful. Pinned by `repository_type_ids_are_stable`.
pub static REPOSITORY_TYPE_ID: &str = "npm";

#[derive(Debug, Default)]
pub struct NpmRegistryType {
    /// Browser-login sessions awaiting approval. In memory on purpose — see [`login::web_login`].
    ///
    /// Owned by the type rather than kept on the application state. `Arc` because both the
    /// registry handlers and the `/api/npm` routes need the same instance, and the routes get
    /// theirs from [`Self::api_router`].
    web_logins: Arc<NpmWebLoginManager>,
}

impl NpmRegistryType {
    pub fn web_logins(&self) -> &Arc<NpmWebLoginManager> {
        &self.web_logins
    }
}

impl RepositoryType for NpmRegistryType {
    fn get_type(&self) -> &'static str {
        REPOSITORY_TYPE_ID
    }

    fn config_types(&self) -> Vec<&str> {
        vec![
            NPMRegistryConfigType::get_type_static(),
            RepositoryPageType::get_type_static(),
            ProjectConfigType::get_type_static(),
        ]
    }

    fn get_description(&self) -> RepositoryTypeDescription {
        RepositoryTypeDescription {
            type_name: REPOSITORY_TYPE_ID,
            name: "NPM",
            description: "A NPM Registry",
            documentation_url: Some("https://nitro-repo.kingtux.dev/repositories/npm/"),
            is_stable: true,
            required_configs: vec![NPMRegistryConfigType::get_type_static()],
        }
    }

    fn create_new(
        &self,
        name: String,
        uuid: uuid::Uuid,
        configs: HashMap<String, serde_json::Value>,
        _storage: nr_storage::DynStorage,
    ) -> BoxFuture<'static, Result<NewRepository, RepositoryFactoryError>> {
        Box::pin(async move {
            let sub_type = configs
                .get(NPMRegistryConfigType::get_type_static())
                .ok_or(RepositoryFactoryError::MissingConfig(
                    NPMRegistryConfigType::get_type_static(),
                ))?
                .clone();
            let _maven_config: NPMRegistryConfig = match serde_json::from_value(sub_type) {
                Ok(ok) => ok,
                Err(err) => {
                    return Err(RepositoryFactoryError::InvalidConfig(
                        NPMRegistryConfigType::get_type_static(),
                        err.to_string(),
                    ));
                }
            };
            Ok(NewRepository {
                name,
                uuid,
                repository_type: REPOSITORY_TYPE_ID.to_string(),
                configs,
            })
        })
    }

    fn load_repo(
        &self,
        repo: DBRepository,
        storage: DynStorage,
        website: SiteContext,
    ) -> BoxFuture<'static, Result<DynRepository, RepositoryFactoryError>> {
        let web_logins = self.web_logins.clone();
        Box::pin(async move {
            let Some(npm_config_db) = DBRepositoryConfig::<NPMRegistryConfig>::get_config(
                repo.id,
                NPMRegistryConfigType::get_type_static(),
                &website.database,
            )
            .await?
            else {
                return Err(RepositoryFactoryError::MissingConfig(
                    NPMRegistryConfigType::get_type_static(),
                ));
            };
            let npm_config = npm_config_db.value.0;
            match npm_config {
                NPMRegistryConfig::Hosted => {
                    let maven_hosted =
                        NPMHostedRegistry::load(website, storage, repo, web_logins).await?;
                    Ok(NPMRegistry::Hosted(maven_hosted).into_dyn())
                }
            }
        })
    }

    /// The browser half of the login exchange, mounted at `/api/npm`.
    ///
    /// Carries the same manager the registries got, so the page that approves a login and the
    /// `npm` process waiting on `doneUrl` are talking about the same sessions.
    fn api_router(&self) -> Option<axum::Router<nr_repository::RepositoryRouterState>> {
        Some(api::npm_routes(self.web_logins.clone()))
    }
}

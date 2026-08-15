//! Docker Registry (OCI distribution) implementation.
//!
//! Speaks the v2 API a Docker daemon, containerd, BuildKit, `crane` and `skopeo` all use:
//! <https://github.com/opencontainers/distribution-spec/blob/main/spec.md>.
//!
//! # Addressing
//!
//! A Docker client cannot be given a URL prefix — `docker pull host/x/y` always requests
//! `https://host/v2/x/y/...`, and there is no setting that changes that. So unlike every other
//! repository type, this one is not reachable only under `/repositories/{storage}/{repository}`.
//! [`routing`] mounts `/v2` at the host root and resolves the repository two ways:
//!
//! * the request's `Host` is registered to a Docker repository — `docker pull
//!   docker.example.com/alpine`;
//! * otherwise the first two segments of the image name are the storage and repository — `docker
//!   pull nitro.example.com/local/docker/alpine`.
//!
//! Both end up in [`dispatch_repository_request`](crate::repository::dispatch_repository_request),
//! the same function the other two entry points use.
//!
//! # What is not here
//!
//! Pull-through caching of an upstream, blob garbage collection, blob deletes and cross-repository
//! blob mounting. Each is called out on the documentation page rather than left to be discovered.

use ahash::HashMap;
use futures::future::BoxFuture;
use hosted::DockerHostedRegistry;
use http::{HeaderName, HeaderValue};
use nr_core::{
    database::entities::repository::{DBRepository, DBRepositoryConfig},
    repository::config::{
        RepositoryConfigType, project::ProjectConfigType, repository_page::RepositoryPageType,
    },
};
use nr_macros::DynRepositoryHandler;
use nr_storage::DynStorage;

pub mod errors;
pub mod hosted;
pub mod routing;
pub mod types;
pub mod uploads;

mod configs;
pub use configs::*;
pub use routing::v2_router;

pub use super::prelude::*;
use super::{DynRepository, NewRepository, RepositoryType, RepositoryTypeDescription};
use crate::{
    app::authentication::AuthenticationError,
    repository::docker::{errors::ErrorCode, uploads::UploadError},
    utils::{IntoErrorResponse, bad_request::BadRequestErrors},
};

/// Every response a registry sends carries this, and older clients use it to decide whether the
/// endpoint speaks v2 at all.
pub static DOCKER_API_VERSION_HEADER: HeaderName =
    HeaderName::from_static("docker-distribution-api-version");
pub static DOCKER_API_VERSION_VALUE: HeaderValue = HeaderValue::from_static("registry/2.0");
/// The digest of the content in the response, which clients verify against.
pub static DOCKER_CONTENT_DIGEST_HEADER: HeaderName =
    HeaderName::from_static("docker-content-digest");
pub static DOCKER_UPLOAD_UUID_HEADER: HeaderName = HeaderName::from_static("docker-upload-uuid");

#[derive(Debug, Clone, DynRepositoryHandler)]
#[repository_handler(error=DockerError)]
pub enum DockerRegistry {
    Hosted(hosted::DockerHostedRegistry),
}

#[derive(Debug, thiserror::Error)]
pub enum DockerError {
    /// Anything that maps cleanly onto one of the spec's error codes, which is most of them.
    #[error("{message}")]
    Coded { code: ErrorCode, message: String },
    #[error("{0}")]
    InvalidDigest(String),
    #[error("{0}")]
    Upload(#[from] UploadError),
    /// The `401` that starts `docker login`: the body is an error envelope and the response also
    /// carries a `WWW-Authenticate` challenge naming where to get a token.
    #[error("authentication is required to access this registry")]
    Challenge {
        realm: String,
        scope: Option<String>,
    },
    #[error("{0}")]
    Other(Box<dyn IntoErrorResponse>),
}

impl DockerError {
    pub fn coded(code: ErrorCode, message: impl Into<String>) -> Self {
        Self::Coded {
            code,
            message: message.into(),
        }
    }
}

impl From<DockerError> for RepositoryHandlerError {
    fn from(err: DockerError) -> Self {
        RepositoryHandlerError::Other(Box::new(err))
    }
}

macro_rules! impl_from_error_for_other {
    ($t:ty) => {
        impl From<$t> for DockerError {
            fn from(e: $t) -> Self {
                DockerError::Other(Box::new(e))
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

/// A path this registry built itself, not one a client sent — every client-supplied component has
/// already been through [`Digest::parse`](types::Digest::parse) or the image-name grammar. So this
/// is a bug rather than a bad request, and `NameInvalid` is the closest code that says so.
impl From<nr_core::storage::InvalidStoragePath> for DockerError {
    fn from(error: nr_core::storage::InvalidStoragePath) -> Self {
        DockerError::coded(ErrorCode::NameInvalid, error.to_string())
    }
}

impl IntoErrorResponse for DockerError {
    fn into_response_boxed(self: Box<Self>) -> Response {
        self.into_response()
    }
}

impl From<DockerError> for DynRepositoryHandlerError {
    fn from(err: DockerError) -> Self {
        DynRepositoryHandlerError(Box::new(err))
    }
}

impl IntoResponse for DockerError {
    fn into_response(self) -> Response {
        match self {
            DockerError::Coded { code, ref message } => errors::oci_error(code, message.clone()),
            DockerError::InvalidDigest(ref message) => {
                errors::oci_error(ErrorCode::DigestInvalid, message.clone())
            }
            DockerError::Upload(ref error) => {
                let code = match error {
                    UploadError::UnknownUpload(_) => ErrorCode::BlobUploadUnknown,
                    UploadError::DigestMismatch { .. } => ErrorCode::DigestInvalid,
                    UploadError::OutOfOrderChunk { .. } => ErrorCode::BlobUploadInvalid,
                    // An IO failure buffering the upload is ours, not the client's, and a 4xx would
                    // tell the client not to retry something a retry might well fix.
                    UploadError::Io(_) => {
                        return errors::oci_error_with_status(
                            StatusCode::INTERNAL_SERVER_ERROR,
                            ErrorCode::BlobUploadInvalid,
                            error.to_string(),
                        );
                    }
                };
                errors::oci_error(code, error.to_string())
            }
            DockerError::Challenge {
                ref realm,
                ref scope,
            } => {
                let mut challenge = format!("Bearer realm=\"{realm}\"");
                if let Some(scope) = scope {
                    challenge.push_str(&format!(",scope=\"{scope}\""));
                }
                let mut response = errors::oci_error_with_status(
                    StatusCode::UNAUTHORIZED,
                    ErrorCode::Unauthorized,
                    self.to_string(),
                );
                if let Ok(value) = HeaderValue::from_str(&challenge) {
                    response
                        .headers_mut()
                        .insert(http::header::WWW_AUTHENTICATE, value);
                }
                response
            }
            DockerError::Other(other) => other.into_response_boxed(),
        }
    }
}

/// The value stored in `repositories.repository_type`. See [`crate::repository::npm::REPOSITORY_TYPE_ID`].
///
/// Also what `routing::resolve` compares a resolved repository's `get_type()` against when deciding
/// whether a `/v2` request is one it should serve.
pub static REPOSITORY_TYPE_ID: &str = "docker";

#[derive(Debug, Default)]
pub struct DockerRegistryType;

impl RepositoryType for DockerRegistryType {
    fn get_type(&self) -> &'static str {
        REPOSITORY_TYPE_ID
    }

    fn config_types(&self) -> Vec<&str> {
        vec![
            DockerRegistryConfigType::get_type_static(),
            RepositoryPageType::get_type_static(),
            ProjectConfigType::get_type_static(),
        ]
    }

    fn get_description(&self) -> RepositoryTypeDescription {
        RepositoryTypeDescription {
            type_name: REPOSITORY_TYPE_ID,
            name: "Docker",
            description: "A Docker registry, speaking the OCI distribution API",
            documentation_url: Some("https://nitro-repo.kingtux.dev/repositories/docker/"),
            is_stable: true,
            required_configs: vec![DockerRegistryConfigType::get_type_static()],
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
                .get(DockerRegistryConfigType::get_type_static())
                .ok_or(RepositoryFactoryError::MissingConfig(
                    DockerRegistryConfigType::get_type_static(),
                ))?
                .clone();
            if let Err(err) = serde_json::from_value::<DockerRegistryConfig>(sub_type) {
                return Err(RepositoryFactoryError::InvalidConfig(
                    DockerRegistryConfigType::get_type_static(),
                    err.to_string(),
                ));
            }
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
        Box::pin(async move {
            let Some(config) = DBRepositoryConfig::<DockerRegistryConfig>::get_config(
                repo.id,
                DockerRegistryConfigType::get_type_static(),
                &website.database,
            )
            .await?
            else {
                return Err(RepositoryFactoryError::MissingConfig(
                    DockerRegistryConfigType::get_type_static(),
                ));
            };
            match config.value.0 {
                DockerRegistryConfig::Hosted => {
                    let hosted = DockerHostedRegistry::load(website, storage, repo).await?;
                    Ok(DockerRegistry::Hosted(hosted).into())
                }
            }
        })
    }
}

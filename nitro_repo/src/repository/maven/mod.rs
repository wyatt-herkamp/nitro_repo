use super::*;
use crate::app::NitroRepo;
use ::http::status::StatusCode;
use ahash::HashMap;
use axum::response::IntoResponse;
pub use configs::*;
use futures::future::BoxFuture;
use hosted::MavenHosted;
use nr_core::{
    builder_error,
    database::repository::{DBRepository, DBRepositoryConfig},
    repository::{
        config::{project::ProjectConfigType, RepositoryConfigType},
        project::ReleaseType,
    },
    storage::StoragePath,
};
use nr_macros::DynRepositoryHandler;
use nr_storage::DynStorage;
use proxy::MavenProxy;
mod configs;
use super::{DynRepository, Repository, RepositoryFactoryError, RepositoryType};
pub mod hosted;
pub mod nitro_deploy;
pub mod proxy;
pub mod utils;
pub static REPOSITORY_TYPE_ID: &str = "maven";
#[derive(Debug, Default)]
pub struct MavenRepositoryType;

impl RepositoryType for MavenRepositoryType {
    fn get_type(&self) -> &'static str {
        REPOSITORY_TYPE_ID
    }

    fn config_types(&self) -> Vec<&str> {
        vec![
            MavenPushRulesConfigType::get_type_static(),
            ProjectConfigType::get_type_static(),
        ]
    }

    fn get_description(&self) -> super::RepositoryTypeDescription {
        super::RepositoryTypeDescription {
            type_name: "maven",
            name: "Maven",
            description: "A Maven Repository",
            documentation_url: Some("https://nitro-repo.kingtux.dev/repositoryTypes/maven/"),
            is_stable: true,
            required_configs: vec![MavenRepositoryConfigType::get_type_static()],
        }
    }

    fn create_new(
        &self,
        name: String,
        uuid: uuid::Uuid,
        configs: HashMap<String, serde_json::Value>,
        storage: nr_storage::DynStorage,
    ) -> BoxFuture<'static, Result<super::NewRepository, super::RepositoryFactoryError>> {
        Box::pin(async move {
            let sub_type = configs
                .get(MavenRepositoryConfigType::get_type_static())
                .ok_or(RepositoryFactoryError::MissingConfig(
                    MavenRepositoryConfigType::get_type_static(),
                ))?
                .clone();
            let maven_config: MavenRepositoryConfig = match serde_json::from_value(sub_type) {
                Ok(ok) => ok,
                Err(err) => {
                    return Err(RepositoryFactoryError::InvalidConfig(
                        MavenRepositoryConfigType::get_type_static(),
                        err.to_string(),
                    ));
                }
            };
            // TODO: Check all configs

            Ok(super::NewRepository {
                name,
                uuid,
                repository_type: "maven".to_string(),
                configs,
            })
        })
    }

    #[doc = " Load a repository from the database"]
    #[doc = " This function should load the repository from the database and return a DynRepository"]
    fn load_repo(
        &self,
        repo: DBRepository,
        storage: DynStorage,
        website: NitroRepo,
    ) -> BoxFuture<'static, Result<DynRepository, RepositoryFactoryError>> {
        Box::pin(async move {
            MavenRepository::load(repo, storage, website)
                .await
                .map(DynRepository::Maven)
        })
    }
}
#[derive(Debug, Clone, DynRepositoryHandler)]
#[repository_handler(error=MavenError)]
pub enum MavenRepository {
    Hosted(MavenHosted),
    Proxy(MavenProxy),
}
impl MavenRepository {
    pub async fn load(
        repo: DBRepository,
        storage: DynStorage,
        website: NitroRepo,
    ) -> Result<Self, RepositoryFactoryError> {
        let Some(maven_config_db) = DBRepositoryConfig::<MavenRepositoryConfig>::get_config(
            repo.id,
            MavenRepositoryConfigType::get_type_static(),
            &website.database,
        )
        .await?
        else {
            return Err(RepositoryFactoryError::MissingConfig(
                MavenRepositoryConfigType::get_type_static(),
            ));
        };
        let maven_config = maven_config_db.value.0;
        match maven_config {
            MavenRepositoryConfig::Hosted => {
                let maven_hosted = MavenHosted::load(repo, storage, website).await?;
                Ok(MavenRepository::Hosted(maven_hosted))
            }
            MavenRepositoryConfig::Proxy(proxy_config) => {
                let proxy = MavenProxy::load(repo, storage, website, proxy_config).await?;
                Ok(MavenRepository::Proxy(proxy))
            }
        }
    }
}
#[derive(Debug, thiserror::Error)]
pub enum MavenError {
    #[error("Error with processing Maven request: {0}")]
    MavenRS(#[from] maven_rs::Error),
    #[error("XML Deserialize Error: {0}")]
    XMLDeserialize(#[from] maven_rs::quick_xml::DeError),
    #[error("Internal Error. {0}")]
    BuilderError(#[from] builder_error::BuilderError),
    #[error("Missing From Pom: {0}")]
    MissingFromPom(&'static str),
    #[error("{0}")]
    Other(Box<dyn IntoErrorResponse>),
}
impl From<MavenError> for DynRepositoryHandlerError {
    fn from(err: MavenError) -> Self {
        DynRepositoryHandlerError(Box::new(err))
    }
}
macro_rules! impl_from_error_for_other {
    ($t:ty) => {
        impl From<$t> for MavenError {
            fn from(e: $t) -> Self {
                MavenError::Other(Box::new(e))
            }
        }
    };
}
impl_from_error_for_other!(BadRequestErrors);
impl_from_error_for_other!(sqlx::Error);
impl_from_error_for_other!(serde_json::Error);
impl_from_error_for_other!(std::io::Error);
impl_from_error_for_other!(AuthenticationError);
impl_from_error_for_other!(RepositoryHandlerError);
impl_from_error_for_other!(nr_storage::StorageError);
impl_from_error_for_other!(reqwest::Error);

impl IntoErrorResponse for MavenError {
    fn into_response_boxed(self: Box<Self>) -> axum::response::Response {
        self.into_response()
    }
}
impl From<MavenError> for RepositoryHandlerError {
    fn from(e: MavenError) -> Self {
        RepositoryHandlerError::Other(Box::new(e))
    }
}

impl IntoResponse for MavenError {
    fn into_response(self) -> axum::http::Response<axum::body::Body> {
        match self {
            MavenError::MavenRS(maven_rs::Error::XMLDeserialize(err))
            | MavenError::XMLDeserialize(err) => axum::http::Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .body(axum::body::Body::from(format!(
                    "XML Deserialize Error: {}",
                    err
                )))
                .unwrap(),
            MavenError::MavenRS(e) => axum::http::Response::builder()
                .status(500)
                .body(axum::body::Body::from(format!("Maven Error: {}", e)))
                .unwrap(),
            err => axum::http::Response::builder()
                .status(500)
                .body(axum::body::Body::from(format!(
                    "Internal Server Error: {}",
                    err
                )))
                .unwrap(),
        }
    }
}
pub fn get_release_type(version: &str) -> ReleaseType {
    let version = version.to_lowercase();
    if version.contains("snapshot") {
        ReleaseType::Snapshot
    } else {
        ReleaseType::Stable
    }
}

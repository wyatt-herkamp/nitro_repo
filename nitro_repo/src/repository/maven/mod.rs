use super::*;
use crate::app::NitroRepo;
use ::http::status::StatusCode;
use ahash::HashMap;
use axum::response::IntoResponse;
use futures::future::LocalBoxFuture;
use hosted::MavenHosted;
use nr_core::{
    database::repository::{DBRepository, GenericDBRepositoryConfig},
    repository::config::{
        frontend::{BadgeSettingsType, FrontendConfigType},
        PushRulesConfigType, RepositoryConfigType as _, SecurityConfigType,
    },
};
use nr_macros::DynRepositoryHandler;
use nr_storage::DynStorage;
use proxy::MavenProxy;
use sqlx::types::Json;

use super::{DynRepository, Repository, RepositoryFactoryError, RepositoryType};
pub mod hosted;
pub mod proxy;
#[derive(Debug, Default)]
pub struct MavenRepositoryType;

impl RepositoryType for MavenRepositoryType {
    fn get_type(&self) -> &'static str {
        "maven"
    }

    fn config_types(&self) -> Vec<&str> {
        vec![
            PushRulesConfigType::get_type_static(),
            SecurityConfigType::get_type_static(),
            BadgeSettingsType::get_type_static(),
            FrontendConfigType::get_type_static(),
        ]
    }

    fn get_description(&self) -> super::RepositoryTypeDescription {
        super::RepositoryTypeDescription {
            type_name: "maven",
            name: "Maven",
            description: "A Maven Repository",
            documentation_url: Some("https://maven.apache.org/"),
            is_stable: true,
            sub_types: vec![
                super::RepositorySubTypeDescription {
                    name: "hosted",
                    description: "A hosted Maven Repository",
                    documentation_url: None,
                    is_stable: true,
                    required_config: &[],
                },
                super::RepositorySubTypeDescription {
                    name: "proxy",
                    description: "A proxy Maven Repository",
                    documentation_url: None,
                    is_stable: true,
                    required_config: &["maven-proxy"],
                },
            ],
        }
    }

    fn create_new(
        &self,
        name: String,
        uuid: uuid::Uuid,
        sub_type: Option<String>,
        configs: HashMap<String, serde_json::Value>,
        storage: nr_storage::DynStorage,
    ) -> LocalBoxFuture<'static, Result<super::NewRepository, super::RepositoryFactoryError>> {
        Box::pin(async move {
            let sub_type: String = if let Some(sub_type) = sub_type {
                if sub_type != "hosted" && sub_type != "proxy" {
                    return Err(super::RepositoryFactoryError::InvalidSubType);
                }
                sub_type
            } else {
                "hosted".to_string()
            };
            if sub_type == "proxy" {
                if !configs.contains_key("maven-proxy") {
                    return Err(super::RepositoryFactoryError::MissingConfig("maven-proxy"));
                }
            }
            let configs: Vec<_> = configs
                .into_iter()
                .map(|(k, v)| GenericDBRepositoryConfig {
                    key: k,
                    repository_id: uuid,
                    value: Json(v),
                    ..Default::default()
                })
                .collect();

            Ok(super::NewRepository {
                name,
                uuid,
                repository_type: "maven".to_string(),
                sub_type: Some(sub_type),
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
    ) -> LocalBoxFuture<'static, Result<DynRepository, RepositoryFactoryError>> {
        Box::pin(async move {
            let sub_type = repo.repository_subtype.clone();

            match sub_type.as_deref() {
                Some("hosted") => {
                    let repo = MavenHosted::load(repo, storage, website).await?;
                    Ok(DynRepository::Maven(MavenRepository::Hosted(repo)))
                }
                Some("proxy") => {
                    todo!()
                }
                _ => Err(RepositoryFactoryError::InvalidSubType),
            }
        })
    }
}
#[derive(Debug, Clone, DynRepositoryHandler)]
pub enum MavenRepository {
    Hosted(MavenHosted),
    Proxy(MavenProxy),
}
#[derive(Debug, thiserror::Error)]
pub enum MavenError {
    #[error("Error with processing Maven request: {0}")]
    MavenRS(#[from] maven_rs::Error),
}
impl IntoResponse for MavenError {
    fn into_response(self) -> axum::http::Response<axum::body::Body> {
        match self {
            MavenError::MavenRS(maven_rs::Error::XMLDeserialize(err)) => {
                axum::http::Response::builder()
                    .status(StatusCode::BAD_REQUEST)
                    .body(axum::body::Body::from(format!(
                        "XML Deserialize Error: {}",
                        err
                    )))
                    .unwrap()
            }
            MavenError::MavenRS(e) => axum::http::Response::builder()
                .status(500)
                .body(axum::body::Body::from(format!("Maven Error: {}", e)))
                .unwrap(),
        }
    }
}

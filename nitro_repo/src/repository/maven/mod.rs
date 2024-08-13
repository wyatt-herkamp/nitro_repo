use super::*;
use crate::app::NitroRepo;
use ::http::status::StatusCode;
use ahash::HashMap;
use axum::response::IntoResponse;
use futures::future::BoxFuture;
use hosted::MavenHosted;
use nr_core::{
    database::repository::{DBRepository, DBRepositoryConfig},
    repository::config::{
        frontend::{BadgeSettingsType, FrontendConfigType},
        PushRulesConfigType, RepositoryConfigError, RepositoryConfigType, SecurityConfigType,
    },
};
use nr_macros::DynRepositoryHandler;
use nr_storage::DynStorage;
use proxy::MavenProxy;
use schemars::{schema_for, JsonSchema};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::{DynRepository, Repository, RepositoryFactoryError, RepositoryType};
pub mod hosted;
pub mod proxy;
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", content = "config")]
pub enum MavenRepositoryConfig {
    Hosted,
}
#[derive(Debug, Clone, Default)]
pub struct MavenRepositoryConfigType;
impl RepositoryConfigType for MavenRepositoryConfigType {
    fn get_type(&self) -> &'static str {
        "maven"
    }

    fn get_type_static() -> &'static str
    where
        Self: Sized,
    {
        "maven"
    }
    fn schema(&self) -> Option<schemars::Schema> {
        Some(schema_for!(MavenRepositoryConfig))
    }
    fn validate_config(&self, config: Value) -> Result<(), RepositoryConfigError> {
        let config: MavenRepositoryConfig = serde_json::from_value(config)?;
        Ok(())
    }

    fn default(&self) -> Result<Value, RepositoryConfigError> {
        let config = MavenRepositoryConfig::Hosted;
        Ok(serde_json::to_value(config).unwrap())
    }
}
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
                .map(|x| DynRepository::Maven(x))
        })
    }
}
#[derive(Debug, Clone, DynRepositoryHandler)]
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
        }
    }
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

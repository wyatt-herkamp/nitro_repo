use super::*;
use crate::app::NitroRepo;
use ::http::status::StatusCode;
use ahash::HashMap;
use axum::response::IntoResponse;
use futures::future::BoxFuture;
use hosted::MavenHosted;
use maven_rs::pom::Pom;
use nr_core::{
    database::{
        project::{
            NewProject, NewProjectBuilder, NewProjectBuilderError, NewVersion, NewVersionBuilder,
            NewVersionBuilderError,
        },
        repository::{DBRepository, DBRepositoryConfig},
    },
    repository::{
        config::{
            frontend::{BadgeSettingsType, FrontendConfigType},
            ConfigDescription, PushRulesConfigType, RepositoryConfigError, RepositoryConfigType,
            SecurityConfigType,
        },
        project::{ReleaseType, VersionDataBuilder, VersionDataBuilderError},
    },
    storage::StoragePath,
};
use nr_macros::DynRepositoryHandler;
use nr_storage::DynStorage;
use proxy::{MavenProxy, MavenProxyConfig};
use schemars::{schema_for, JsonSchema};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use super::{DynRepository, Repository, RepositoryFactoryError, RepositoryType};
pub mod hosted;
pub mod nitro_deploy;
pub mod proxy;
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", content = "config")]
pub enum MavenRepositoryConfig {
    Hosted,
    Proxy { proxy_config: MavenProxyConfig },
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
    fn get_description(&self) -> ConfigDescription {
        ConfigDescription {
            name: "Maven Repository Config",
            description: Some("Handles the type of Maven Repository"),
            documentation_link: None,
            ..Default::default()
        }
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
            MavenRepositoryConfig::Proxy { proxy_config } => {
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
    #[error("Database Error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("New Project Error: {0}")]
    NewProject(#[from] NewProjectBuilderError),
    #[error("New Version Error: {0}")]
    NewVersion(#[from] NewVersionBuilderError),
    #[error("New Version Error: {0}")]
    VersionData(#[from] VersionDataBuilderError),
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

pub fn pom_to_db_project(
    project_path: StoragePath,
    repository: Uuid,
    pom: Pom,
) -> Result<NewProject, MavenError> {
    let result = NewProjectBuilder::default()
        .project_key(format!("{}:{}", pom.group_id, pom.artifact_id))
        .scope(Some(pom.group_id))
        .name(pom.name.unwrap_or(pom.artifact_id))
        .description(pom.description)
        .repository(repository)
        .storage_path(project_path.to_string())
        .build()?;
    Ok(result)
}
pub fn pom_to_db_project_version(
    project_id: Uuid,
    version_path: StoragePath,
    publisher: i32,
    pom: Pom,
) -> Result<NewVersion, MavenError> {
    let version_data = VersionDataBuilder::default()
        .description(pom.description)
        .build()?;
    let release_type = ReleaseType::release_type_from_version(&pom.version);
    let result = NewVersionBuilder::default()
        .project_id(project_id)
        .version(pom.version)
        .publisher(publisher)
        .version_path(version_path.to_string())
        .release_type(release_type)
        .extra(version_data)
        .build()?;
    Ok(result)
}

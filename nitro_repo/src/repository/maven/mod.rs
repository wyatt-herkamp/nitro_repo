use ahash::HashMap;
use futures::future::LocalBoxFuture;
use hosted::MavenHosted;
use nr_core::database::repository::{DBRepository, GenericDBRepositoryConfig};
use nr_macros::DynRepositoryHandler;
use nr_storage::DynStorage;
use proxy::MavenProxy;
use sqlx::types::Json;

use crate::app::NitroRepo;

use super::{DynRepository, Repository, RepositoryFactoryError, RepositoryType};
pub mod hosted;
pub mod proxy;
#[derive(Debug, Default)]
pub struct MavenRepositoryType;

impl RepositoryType for MavenRepositoryType {
    fn get_type(&self) -> &'static str {
        "maven"
    }

    fn config_types(&self) -> &'static [&'static str] {
        &["push_rules", "security"]
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
        todo!()
    }
}
#[derive(Debug, Clone, DynRepositoryHandler)]
pub enum MavenRepository {
    Hosted(MavenHosted),
    Proxy(MavenProxy),
}

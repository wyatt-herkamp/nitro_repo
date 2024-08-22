use ahash::HashMap;
use futures::future::BoxFuture;
use nr_core::database::repository::DBRepository;
use nr_storage::DynStorage;

use crate::{
    app::NitroRepo,
    repository::{
        DynRepository, NewRepository, RepositoryFactoryError, RepositoryType,
        RepositoryTypeDescription,
    },
};

use super::NpmRegistry;

#[derive(Debug, Default)]
pub struct NpmRegistryType;

impl RepositoryType for NpmRegistryType {
    fn get_type(&self) -> &'static str {
        "npm"
    }

    fn config_types(&self) -> Vec<&str> {
        vec![]
    }

    fn get_description(&self) -> RepositoryTypeDescription {
        RepositoryTypeDescription {
            type_name: "npm",
            name: "NPM",
            description: "A NPM Registry",
            documentation_url: None,
            is_stable: true,
            required_configs: vec![],
        }
    }

    fn create_new(
        &self,
        name: String,
        uuid: uuid::Uuid,
        configs: HashMap<String, serde_json::Value>,
        storage: nr_storage::DynStorage,
    ) -> BoxFuture<'static, Result<NewRepository, RepositoryFactoryError>> {
        Box::pin(async move {
            Ok(NewRepository {
                name,
                uuid,
                repository_type: "npm".to_string(),
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
            NpmRegistry::load(website, storage, repo)
                .await
                .map(DynRepository::from)
        })
    }
}

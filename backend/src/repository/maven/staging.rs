use crate::repository::handler::RepositoryHandler;
use crate::repository::maven::models::ProxySettings;
use crate::repository::nitro::nitro_repository::NitroRepositoryHandler;
use crate::repository::settings::RepositoryConfig;
use crate::storage::models::Storage;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLockReadGuard;
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum StageSettings {
    InternalRepository {
        storage: String,
        repository: String,
    },
    // The Po mans Repository. (For https://github.com/NickAcPT)
    GitPush {
        git_url: String,
        git_branch: String,
        git_username: String,
        git_password: String,
    },
    ExternalRepository {
        repository: String,
        username: String,
        password: String,
    },
}
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum DeployRequirement {}
pub struct StagingRepository<'a, S: Storage> {
    pub config: RepositoryConfig,
    pub storage: RwLockReadGuard<'a, S>,
    pub stage_to: Vec<StageSettings>,
    pub deploy_requirement: Vec<DeployRequirement>,
}
#[async_trait]
impl<'a, S: Storage> RepositoryHandler<'a, S> for StagingRepository<'a, S> {}
impl<StorageType: Storage> NitroRepositoryHandler<StorageType>
    for StagingRepository<'_, StorageType>
{
    fn parse_project_to_directory<S: Into<String>>(value: S) -> String {
        value.into().replace('.', "/").replace(':', "/")
    }

    fn storage(&self) -> &StorageType {
        &self.storage
    }

    fn repository(&self) -> &RepositoryConfig {
        &self.config
    }
}

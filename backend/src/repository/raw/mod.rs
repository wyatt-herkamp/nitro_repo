use crate::error::internal_error::InternalError;
use crate::repository::handler::{CreateRepository, Repository, RepositoryType};
use crate::repository::settings::{RepositoryConfig, RepositoryConfigType};
use crate::storage::models::Storage;
use crate::utils::get_current_time;
use async_trait::async_trait;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug)]
pub struct RawHandler<StorageType: Storage> {
    config: RepositoryConfig,
    storage: Arc<StorageType>,
}
impl<StorageType: Storage> RawHandler<StorageType> {
    pub async fn create(
        config: RepositoryConfig,
        storage: Arc<StorageType>,
    ) -> Result<RawHandler<StorageType>, InternalError> {
        Ok(RawHandler { config, storage })
    }
}
impl<S: Storage> Clone for RawHandler<S> {
    fn clone(&self) -> Self {
        RawHandler {
            config: self.config.clone(),
            storage: self.storage.clone(),
        }
    }
}
crate::repository::settings::define_configs_on_handler!(RawHandler<StorageType>);

#[async_trait]
impl<StorageType: Storage> Repository<StorageType> for RawHandler<StorageType> {
    fn get_repository(&self) -> &RepositoryConfig {
        &self.config
    }
    fn get_mut_config(&mut self) -> &mut RepositoryConfig {
        &mut self.config
    }
    fn get_storage(&self) -> &StorageType {
        &self.storage
    }
}
#[derive(Debug, Serialize, Deserialize, Clone, Default, JsonSchema)]
pub struct RawSettings {}

impl RepositoryConfigType for RawSettings {

    fn config_name() -> &'static str {
        "raw.json"
    }
}

impl<S: Storage> CreateRepository<S> for RawHandler<S> {
    type Config = RawSettings;
    type Error = InternalError;

    fn create_repository(
        config: Self::Config,
        name: impl Into<String>,
        storage: Arc<S>,
    ) -> Result<(Self, Self::Config), Self::Error>
    where
        Self: Sized,
    {
        let repository_config = RepositoryConfig {
            name: name.into(),
            repository_type: RepositoryType::Raw,
            storage: storage.storage_config().generic_config.id.clone(),
            visibility: Default::default(),
            active: true,
            require_token_over_basic: false,
            created: get_current_time(),
        };
        Ok((
            RawHandler {
                config: repository_config,
                storage,
            },
            config,
        ))
    }
}

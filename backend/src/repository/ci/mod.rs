use crate::error::internal_error::InternalError;
use crate::repository::handler::{CreateRepository, Repository};
use crate::repository::settings::badge::BadgeSettings;
use crate::repository::settings::frontend::Frontend;
use crate::repository::settings::{RepositoryConfig, RepositoryConfigType, RepositoryType};
use crate::storage::models::Storage;
use crate::utils::get_current_time;
use async_trait::async_trait;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
pub struct CISettings {}
impl RepositoryConfigType for CISettings {

    fn config_name() -> &'static str {
        "ci.json"
    }
}
#[derive(Debug)]
pub struct CIHandler<StorageType: Storage> {
    config: RepositoryConfig,
    storage: Arc<StorageType>,
    badge: BadgeSettings,
    frontend: Frontend,
}
impl<StorageType: Storage> CIHandler<StorageType> {
    pub async fn create(
        config: RepositoryConfig,
        storage: Arc<StorageType>,
    ) -> Result<CIHandler<StorageType>, InternalError> {
        Ok(CIHandler {
            config,
            storage,
            badge: Default::default(),
            frontend: Default::default(),
        })
    }
}
impl<S: Storage> Clone for CIHandler<S> {
    fn clone(&self) -> Self {
        CIHandler {
            config: self.config.clone(),
            storage: self.storage.clone(),
            badge: self.badge.clone(),
            frontend: self.frontend.clone(),
        }
    }
}

crate::repository::settings::define_configs_on_handler!(
    CIHandler<StorageType>,
    badge,
    BadgeSettings,
    frontend,
    Frontend
);

impl<S: Storage> CreateRepository<S> for CIHandler<S> {
    type Config = CISettings;
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
            repository_type: RepositoryType::CI,
            storage: storage.storage_config().generic_config.id.clone(),
            visibility: Default::default(),
            active: true,
            require_token_over_basic: false,
            created: get_current_time(),
        };
        Ok((
            CIHandler {
                config: repository_config,
                storage: storage.clone(),
                badge: Default::default(),
                frontend: Default::default(),
            },
            config,
        ))
    }
}
#[async_trait]
impl<StorageType: Storage> Repository<StorageType> for CIHandler<StorageType> {
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

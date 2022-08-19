use crate::error::internal_error::InternalError;
use crate::repository::handler::Repository;
use crate::repository::settings::RepositoryConfig;
use crate::storage::models::Storage;
use async_trait::async_trait;
use std::sync::Arc;
#[derive(Debug)]
pub struct DockerHandler<StorageType: Storage> {
    config: RepositoryConfig,
    storage: Arc<StorageType>,
}
impl<StorageType: Storage> DockerHandler<StorageType> {
    pub async fn create(
        config: RepositoryConfig,
        storage: Arc<StorageType>,
    ) -> Result<DockerHandler<StorageType>, InternalError> {
        Ok(DockerHandler { config, storage })
    }
}

impl<StorageType: Storage> Clone for DockerHandler<StorageType> {
    fn clone(&self) -> Self {
        DockerHandler {
            config: self.config.clone(),
            storage: self.storage.clone(),
        }
    }
}
crate::repository::settings::define_configs_on_handler!(DockerHandler<StorageType>);

#[async_trait]
impl<StorageType: Storage> Repository<StorageType> for DockerHandler<StorageType> {
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

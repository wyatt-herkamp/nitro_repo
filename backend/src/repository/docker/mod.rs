use crate::error::internal_error::InternalError;
use crate::repository::handler::Repository;
use crate::repository::settings::RepositoryConfig;
use crate::storage::models::Storage;
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::RwLockReadGuard;

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

#[async_trait]
impl<StorageType: Storage> Repository<StorageType> for DockerHandler<StorageType> {
    fn get_repository(&self) -> &RepositoryConfig {
        &self.config
    }

    fn get_storage(&self) -> &StorageType {
        &self.storage
    }
}

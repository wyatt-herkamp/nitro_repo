use crate::repository::handler::RepositoryHandler;
use crate::repository::settings::RepositoryConfig;
use crate::storage::models::Storage;
use async_trait::async_trait;
use tokio::sync::RwLockReadGuard;
pub struct DockerHandler<'a, StorageType: Storage> {
    config: RepositoryConfig,
    storage: RwLockReadGuard<'a, StorageType>,
}
impl<'a, StorageType: Storage> DockerHandler<'a, StorageType> {
    pub fn create(config: RepositoryConfig, storage: RwLockReadGuard<'a, StorageType>) -> Self {
        DockerHandler { config, storage }
    }
}

#[async_trait]
impl<'a, StorageType: Storage> RepositoryHandler<'a, StorageType>
    for DockerHandler<'a, StorageType>
{
}

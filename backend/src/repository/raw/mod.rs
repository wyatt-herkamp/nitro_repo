use crate::repository::handler::RepositoryHandler;
use crate::repository::settings::RepositoryConfig;
use crate::storage::models::Storage;
use async_trait::async_trait;
use tokio::sync::RwLockReadGuard;
pub struct RawHandler<'a, StorageType: Storage> {
    config: RepositoryConfig,
    storage: RwLockReadGuard<'a, StorageType>,
}
impl<'a, StorageType: Storage> RawHandler<'a, StorageType> {
    pub fn create(config: RepositoryConfig, storage: RwLockReadGuard<'a, StorageType>) -> Self {
        RawHandler { config, storage }
    }
}

#[async_trait]
impl<'a, StorageType: Storage> RepositoryHandler<'a, StorageType> for RawHandler<'a, StorageType> {}

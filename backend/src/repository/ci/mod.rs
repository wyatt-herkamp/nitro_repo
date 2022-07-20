use crate::error::internal_error::InternalError;
use crate::repository::handler::RepositoryHandler;
use crate::repository::settings::RepositoryConfig;
use crate::storage::models::Storage;
use async_trait::async_trait;
use tokio::sync::RwLockReadGuard;

pub struct CIHandler<'a, StorageType: Storage> {
    config: RepositoryConfig,
    storage: RwLockReadGuard<'a, StorageType>,
}
impl<'a, StorageType: Storage> CIHandler<'a, StorageType> {
    pub async fn create(
        config: RepositoryConfig,
        storage: RwLockReadGuard<'a, StorageType>,
    ) -> Result<CIHandler<'a, StorageType>, InternalError> {
        Ok(CIHandler { config, storage })
    }
}

#[async_trait]
impl<'a, StorageType: Storage> RepositoryHandler<'a, StorageType> for CIHandler<'a, StorageType> {}

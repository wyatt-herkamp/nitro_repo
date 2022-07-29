use crate::error::internal_error::InternalError;
use crate::repository::handler::Repository;
use crate::repository::settings::RepositoryConfig;
use crate::storage::models::Storage;
use async_trait::async_trait;
use std::sync::Arc;
#[derive(Debug)]
pub struct CIHandler<StorageType: Storage> {
    config: RepositoryConfig,
    storage: Arc<StorageType>,
}
impl<StorageType: Storage> CIHandler<StorageType> {
    pub async fn create(
        config: RepositoryConfig,
        storage: Arc<StorageType>,
    ) -> Result<CIHandler<StorageType>, InternalError> {
        Ok(CIHandler { config, storage })
    }
}
impl<S: Storage> Clone for CIHandler<S> {
    fn clone(&self) -> Self {
        CIHandler {
            config: self.config.clone(),
            storage: self.storage.clone(),
        }
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

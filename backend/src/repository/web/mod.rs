use serde::Serialize;

use crate::error::internal_error::InternalError;
use crate::repository::settings::frontend::{BadgeSettings, Frontend};
use crate::repository::settings::RepositoryConfig;
use crate::storage::models::Storage;

pub mod multi;

#[derive(Serialize, Debug)]
pub struct RepositoryResponse<'a> {
    pub config: &'a RepositoryConfig,
    pub frontend: Option<Frontend>,
    pub badge: Option<BadgeSettings>,
}

impl<'a> RepositoryResponse<'a> {
    pub async fn new<StorageType: Storage>(
        config: &'a RepositoryConfig,
        storage: &StorageType,
    ) -> Result<RepositoryResponse<'a>, InternalError> {
        let frontend = config.get_config(storage).await?;
        let badge = config.get_config(storage).await?;
        Ok(RepositoryResponse {
            config,
            frontend,
            badge,
        })
    }
}

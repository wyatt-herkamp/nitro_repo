use serde::Serialize;

use crate::error::internal_error::InternalError;
use crate::repository::data::RepositoryConfig;
use crate::repository::settings::frontend::{BadgeSettings, Frontend};
use crate::storage::models::Storage;

pub mod multi;

#[derive(Serialize, Debug)]
pub struct RepositoryResponse<'a> {
    pub config: &'a RepositoryConfig,
    pub frontend: Option<Frontend>,
    pub badge: Option<BadgeSettings>,
}

impl<'a> RepositoryResponse<'a> {
    pub async fn new(
        config: &'a RepositoryConfig,
        storage: &dyn Storage,
    ) -> Result<RepositoryResponse<'a>, InternalError> {
        let frontend = config.get_frontend_config(storage).await?;
        let badge: Option<BadgeSettings> = None;
        Ok(RepositoryResponse {
            config,
            frontend,
            badge,
        })
    }
}

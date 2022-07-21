use serde::Serialize;

use crate::error::internal_error::InternalError;
use crate::repository::settings::badge::BadgeSettings;
use crate::repository::settings::frontend::Frontend;
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
        _config: &'a RepositoryConfig,
    ) -> Result<RepositoryResponse<'a>, InternalError> {
        todo!("create big boi")
    }
}

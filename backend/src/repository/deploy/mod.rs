use std::fmt::{Display, Formatter};

use serde::{Deserialize, Serialize};

use crate::error::internal_error::InternalError;
use crate::repository::data::RepositoryDataType;
use crate::storage::models::Storage;
use crate::system::user::UserModel;

#[derive(Clone, Deserialize, Serialize)]
pub struct DeployInfo {
    pub user: UserModel,
    pub version: String,
    pub name: String,
    pub version_folder: String,
}

impl Display for DeployInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", serde_json::to_string(self).unwrap())
    }
}

pub async fn handle_post_deploy<R: RepositoryDataType>(
    _storage: &Storage,
    _repository: &R,
    _deploy: &DeployInfo,
) -> Result<(), InternalError> {
    todo!("Not Implemented");
    Ok(())
}

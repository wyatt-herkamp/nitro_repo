pub mod dynamic;

use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::{
    error::internal_error::InternalError,
    repository::settings::RepositoryConfigType,
    storage::{multi::MultiStorageController, DynamicStorage, Storage},
    system::user::database::UserSafeData,
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProcessingStage {
    pub files: Vec<String>,
}
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProjectsToStage {
    pub projects: Vec<String>,
}
impl RepositoryConfigType for ProcessingStage {
    fn config_name() -> &'static str {
        "to_stage.json"
    }
}
pub trait StageHandler<S: Storage> {
    fn staging_repository(&self) -> bool {
        true
    }

    async fn push(
        &self,
        directory: String,
        storages: Arc<MultiStorageController<DynamicStorage>>,
        user: UserSafeData,
    ) -> Result<(), InternalError>;
}

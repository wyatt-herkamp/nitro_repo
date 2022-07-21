pub mod dynamic;

use crate::error::internal_error::InternalError;
use crate::repository::response::RepoResponse;
use crate::repository::settings::RepositoryConfigType;
use crate::storage::models::Storage;
use crate::storage::multi::MultiStorageController;
use crate::storage::DynamicStorage;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::PathBuf;
use std::sync::{Arc, Weak};

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
#[async_trait]
pub trait StageHandler<S: Storage> {
    async fn push(
        &self,
        directory: String,
        process: ProcessingStage,
        storages: Arc<MultiStorageController<DynamicStorage>>,
    ) -> Result<(), InternalError>;
}

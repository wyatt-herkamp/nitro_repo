use std::fmt::Debug;

use serde::{Deserialize, Serialize};

use crate::repository::settings::security::SecurityRules;
use crate::repository::settings::webhook::DeploySettings;
use crate::repository::settings::RepositorySettings;
use crate::repository::types::RepositoryType;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RepositorySummary {
    pub name: String,
    pub storage: String,
    pub repo_type: String,
}

impl RepositorySummary {
    pub fn new(repo: &Repository) -> RepositorySummary {
        RepositorySummary {
            name: repo.name.clone(),
            storage: repo.storage.clone(),
            repo_type: repo.repo_type.to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Repository {
    pub name: String,
    pub repo_type: RepositoryType,
    pub storage: String,
    pub settings: RepositorySettings,
    pub security: SecurityRules,
    pub deploy_settings: DeploySettings,
    pub created: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryListResponse {
    pub name: String,
    pub repo_type: String,
    pub storage: String,
}

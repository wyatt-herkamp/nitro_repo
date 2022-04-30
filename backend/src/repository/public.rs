use actix_web::{get, web, HttpRequest};
use sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};

use crate::api_response::{APIResponse, SiteResponse};
use crate::authentication::Authentication;
use crate::error::response::not_found;
use crate::repository::models::Repository;
use crate::repository::settings::security::Visibility;
use crate::repository::settings::Policy;
use crate::system::permissions::options::CanIDo;
use crate::system::user::UserModel;
use crate::NitroRepoData;
use crate::storage::multi::MultiStorageController;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicRepositoryResponse {
    pub name: String,
    pub repo_type: String,
    pub storage: String,
    pub description: String,
    pub active: bool,
    pub visibility: Visibility,
    pub policy: Policy,
    pub created: i64,
}

impl From<Repository> for PublicRepositoryResponse {
    fn from(repo: Repository) -> Self {
        PublicRepositoryResponse {
            name: repo.name,
            repo_type: repo.repo_type.to_string(),
            storage: repo.storage,
            description: repo.settings.description,
            active: repo.settings.active,
            visibility: repo.security.visibility,
            policy: repo.settings.policy,
            created: repo.created,
        }
    }
}


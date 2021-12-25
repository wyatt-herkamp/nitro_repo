use std::fs::create_dir_all;
use std::path::PathBuf;

use actix_web::{delete, get, HttpRequest, patch, post, put, web};
use actix_web::web::Bytes;
use serde::{Deserialize, Serialize};

use crate::api_response::{APIResponse, SiteResponse};
use crate::DbPool;
use crate::error::response::{already_exists, bad_request, not_found, unauthorized};
use crate::repository::action::{
    add_new_repository, get_repo_by_id, get_repo_by_name_and_storage, get_repositories,
    update_deploy_settings,
};
use crate::repository::action::{update_repo_security, update_repo_settings};
use crate::repository::models::{
    BadgeSettings, Frontend, Policy, Repository, RepositoryListResponse, RepositorySettings,
    SecurityRules, Visibility,
};
use crate::repository::models::{ReportGeneration, Webhook};
use crate::storage::action::get_storage_by_name;
use crate::system::action::get_user_by_username;
use crate::system::utils::{can_read_basic_auth, get_user_by_header};
use crate::utils::get_current_time;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicRepositoryResponse {
    pub id: i64,
    pub name: String,
    pub repo_type: String,
    pub storage: String,
    pub description: String,
    pub active: bool,
    pub policy: Policy,
    pub created: i64,
}

impl From<Repository> for PublicRepositoryResponse {
    fn from(repo: Repository) -> Self {
        PublicRepositoryResponse {
            id: repo.id,
            name: repo.name,
            repo_type: repo.repo_type,
            storage: repo.storage,
            description: repo.settings.description,
            active: repo.settings.active,
            policy: repo.settings.policy,
            created: repo.created,
        }
    }
}

#[get("/api/repositories/get/{storage}/{repo}")]
pub async fn get_repo(
    pool: web::Data<DbPool>,
    r: HttpRequest,
    path: web::Path<(String, String)>,
) -> SiteResponse {
    let (storage, repo) = path.into_inner();
    let connection = pool.get()?;

    let repo = get_repo_by_name_and_storage(&repo, &storage, &connection)?;
    if repo.is_none() {
        return not_found();
    }
    let repository = repo.unwrap();
    if !can_read_basic_auth(&r.headers(), &repository, &connection)? {
        return unauthorized();
    }

    APIResponse::respond_new(Some(PublicRepositoryResponse::from(repository)), &r)
}
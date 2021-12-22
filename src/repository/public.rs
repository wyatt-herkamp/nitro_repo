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
use crate::system::utils::get_user_by_header;
use crate::utils::get_current_time;

#[get("/api/repositories/get/{storage}/{repo}")]
pub async fn get_repo(
    pool: web::Data<DbPool>,
    r: HttpRequest,
    path: web::Path<(String, String)>,
) -> SiteResponse {
    let (storage, repo) = path.into_inner();
    let connection = pool.get()?;
    let user = get_user_by_header(r.headers(), &connection)?;
    let storage = get_storage_by_name(&storage, &connection)?;
    if storage.is_none() {
        return not_found();
    }
    let repo = get_repo_by_name_and_storage(&repo, &storage.unwrap().id, &connection)?;

    APIResponse::respond_new(repo, &r)
}
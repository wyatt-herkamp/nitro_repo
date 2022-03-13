use actix_web::{get, web, HttpRequest};
use serde::{Deserialize, Serialize};

use crate::api_response::{APIResponse, SiteResponse};
use crate::error::response::{not_found, unauthorized};
use crate::repository::action::get_repo_by_name_and_storage;
use crate::repository::models::{Policy, Repository, Visibility};
use crate::system::utils::can_read_basic_auth;
use crate::DbPool;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicRepositoryResponse {
    pub id: i64,
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
            id: repo.id,
            name: repo.name,
            repo_type: repo.repo_type,
            storage: repo.storage,
            description: repo.settings.description,
            active: repo.settings.active,
            visibility: repo.security.visibility,
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
    if !can_read_basic_auth(r.headers(), &repository, &connection)? {
        return unauthorized();
    }

    APIResponse::respond_new(Some(PublicRepositoryResponse::from(repository)), &r)
}

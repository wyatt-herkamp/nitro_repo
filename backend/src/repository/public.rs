use actix_web::{get, web, HttpRequest};
use serde::{Deserialize, Serialize};

use crate::api_response::{APIResponse, SiteResponse};
use crate::database::DbPool;
use crate::error::response::{not_found, unauthorized};
use crate::repository::models::{Policy, Repository, Visibility};
use crate::system::utils::can_read_basic_auth;
use crate::NitroRepoData;

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
    site: NitroRepoData,
    r: HttpRequest,
    path: web::Path<(String, String)>,
) -> SiteResponse {
    let (storage, repo) = path.into_inner();
    let guard = site.storages.lock().unwrap();
    if let Some(storage) = guard.get(&storage) {
        let option = storage.get_repository(&repo)?;
        if let Some(repository) = option {
            if repository.security.visibility.eq(&Visibility::Private) {
                let connection = pool.get()?;
                if !can_read_basic_auth(r.headers(), &repository, &connection)? {
                    return unauthorized();
                }
            }
            return APIResponse::respond_new(Some(PublicRepositoryResponse::from(repository)), &r);
        }
    }
    not_found()
}

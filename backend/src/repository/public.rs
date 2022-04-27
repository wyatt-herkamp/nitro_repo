use actix_web::{get, web, HttpRequest};
use sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};

use crate::api_response::{APIResponse, SiteResponse};
use crate::error::response::{not_found, unauthorized};
use crate::repository::models::{Repository};
use crate::system::utils::can_read_basic_auth;
use crate::NitroRepoData;
use crate::repository::settings::Policy;
use crate::repository::settings::security::Visibility;

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

#[get("/api/repositories/get/{storage}/{repo}")]
pub async fn get_repo(
    connection: web::Data<DatabaseConnection>,
    site: NitroRepoData,
    r: HttpRequest,
    path: web::Path<(String, String)>,
) -> SiteResponse {
    let (storage, repo) = path.into_inner();
    let guard = site.storages.read().await;
    if let Some(storage) = guard.get(&storage) {
        let option = storage.get_repository(&repo)?;
        if let Some(repository) = option {
            if repository.security.visibility.eq(&Visibility::Private) {
                if !can_read_basic_auth(r.headers(), &repository, &connection).await?.0 {
                    return unauthorized();
                }
            }
            return APIResponse::respond_new(Some(PublicRepositoryResponse::from(repository)), &r);
        }
    }
    not_found()
}

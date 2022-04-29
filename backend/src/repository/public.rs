use actix_web::{get, web, HttpRequest};
use sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};

use crate::api_response::{APIResponse, SiteResponse};
use crate::authentication::Authentication;
use crate::error::response::not_found;
use crate::repository::models::Repository;
use crate::repository::settings::security::Visibility;
use crate::repository::settings::Policy;
use crate::storage::{StorageHandlerType, StorageManager};
use crate::system::permissions::options::CanIDo;
use crate::system::user::UserModel;
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
    _site: NitroRepoData,
    storages: web::Data<StorageManager>,
    r: HttpRequest,
    auth: Authentication,
    path: web::Path<(String, String)>,
) -> SiteResponse {
    let (storage, repo) = path.into_inner();
    if let Some(storage) = storages.get_storage_by_name(&storage).await? {
        let option = storage.get_repository(&repo).await?;
        if let Some(repository) = option {
            if repository.security.visibility.eq(&Visibility::Private) {
                let caller: UserModel = auth.get_user(&connection).await??;
                caller.can_read_from(&repository)?;
            }
            return APIResponse::respond_new(Some(PublicRepositoryResponse::from(repository)), &r);
        }
    }
    not_found()
}

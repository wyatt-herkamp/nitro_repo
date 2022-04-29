use actix_web::web::{Bytes, Path};
use actix_web::{delete, get, patch, post, put, web, HttpRequest};
use log::error;
use sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};

use crate::api_response::{APIResponse, SiteResponse};
use crate::constants::SUPPORTED_REPO_TYPES;
use crate::error::response::{bad_request, not_found};
use crate::repository::models::RepositorySummary;
use crate::repository::settings::frontend::{BadgeSettings, Frontend};
use crate::repository::settings::security::Visibility;
use crate::repository::settings::webhook::{ReportGeneration, Webhook};
use crate::repository::settings::Policy;
use crate::system::permissions::options::CanIDo;
use crate::NitroRepoData;
use crate::authentication::Authentication;
use crate::storage::{StorageHandlerType, StorageManager};
use crate::system::user::UserModel;


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListRepositories {
    pub repositories: Vec<RepositorySummary>,
}


#[get("/api/admin/repositories/{storage}/list")]
pub async fn list_repos_by_storage(
    connection: web::Data<DatabaseConnection>,
    site: NitroRepoData,
    r: HttpRequest, auth: Authentication, storages: web::Data<StorageManager>,
    storage: Path<String>,
) -> SiteResponse {
    let caller: UserModel = auth.get_user(&connection).await??;
    caller.can_i_edit_repos()?;
    let storage = storage.into_inner();
    let storage = storages.get_storage_by_name(&storage).await?;
    if storage.is_none() {
        return not_found();
    }
    let storage = storage.unwrap();
    let mut vec = Vec::new();
    for (_, repo) in storage.get_repositories().await? {
        vec.push(repo);
    }
    let response = ListRepositories { repositories: vec };
    APIResponse::new(true, Some(response)).respond(&r)
}

#[get("/api/admin/repositories/get/{storage}/{repo}")]
pub async fn get_repo(
    connection: web::Data<DatabaseConnection>,
    site: NitroRepoData,
    r: HttpRequest, auth: Authentication, storages: web::Data<StorageManager>,
    path: web::Path<(String, String)>,
) -> SiteResponse {
    let (storage, repo) = path.into_inner();

    let caller: UserModel = auth.get_user(&connection).await??;
    caller.can_i_edit_repos()?;
    let storage = storages.get_storage_by_name(&storage).await?;
    if storage.is_none() {
        return not_found();
    }
    let storage = storage.unwrap();
    let repository = storage.get_repository(&repo).await?;
    APIResponse::from(repository).respond(&r)
}

#[post("/api/admin/repository/add")]
pub async fn add_repo(
    connection: web::Data<DatabaseConnection>,
    site: NitroRepoData, auth: Authentication, storages: web::Data<StorageManager>,
    r: HttpRequest,
    nc: web::Json<RepositorySummary>,
) -> SiteResponse {
    let caller: UserModel = auth.get_user(&connection).await??;
    caller.can_i_edit_repos()?;
    let storage = storages.get_storage_by_name(&nc.storage).await?;
    if storage.is_none() {
        return not_found();
    }
    if !SUPPORTED_REPO_TYPES.contains(&nc.repo_type.as_str()) {
        return bad_request(format!("Unsupported Repository Type {}", &nc.repo_type));
    }
    let storage = storage.unwrap();
    let repository = storage.create_repository(nc.0).await?;

    APIResponse::from(Some(repository)).respond(&r)
}

#[patch("/api/admin/repository/{storage}/{repository}/active/{active}")]
pub async fn update_active_status(
    connection: web::Data<DatabaseConnection>,
    site: NitroRepoData,
    r: HttpRequest, auth: Authentication, storages: web::Data<StorageManager>,
    path: web::Path<(String, String, bool)>,
) -> SiteResponse {
    let caller: UserModel = auth.get_user(&connection).await??;
    caller.can_i_edit_repos()?;

    let (storage, repo, active) = path.into_inner();
    let storage = storages.get_storage_by_name(&storage).await?;
    if storage.is_none() {
        return not_found();
    }
    let storage = storage.unwrap();
    let repository = storage.get_repository(&repo).await?;
    if repository.is_none() {
        return not_found();
    }
    let mut repository = repository.unwrap();
    repository.settings.active = active;
    storage.update_repository(&repository).await?;
    APIResponse::new(true, Some(repository)).respond(&r)
}

#[patch("/api/admin/repository/{storage}/{repository}/policy/{policy}")]
pub async fn update_policy(
    connection: web::Data<DatabaseConnection>,
    site: NitroRepoData,
    r: HttpRequest, auth: Authentication, storages: web::Data<StorageManager>,
    path: web::Path<(String, String, Policy)>,
) -> SiteResponse {
    let caller: UserModel = auth.get_user(&connection).await??;
    caller.can_i_edit_repos()?;

    let (storage, repository, policy) = path.into_inner();
    let storage = storages.get_storage_by_name(&storage).await?;
    if storage.is_none() {
        return not_found();
    }
    let storage = storage.unwrap();
    let repository = storage.get_repository(&repository).await?;
    if repository.is_none() {
        return not_found();
    }
    let mut repository = repository.unwrap();
    repository.settings.policy = policy;
    storage.update_repository(&repository).await?;
    APIResponse::new(true, Some(repository)).respond(&r)
}

#[patch("/api/admin/repository/{storage}/{repository}/description")]
pub async fn update_description(
    connection: web::Data<DatabaseConnection>, storages: web::Data<StorageManager>,
    site: NitroRepoData,
    r: HttpRequest, auth: Authentication,
    b: Bytes,
    path: web::Path<(String, String)>,
) -> SiteResponse {
    let caller: UserModel = auth.get_user(&connection).await??;
    caller.can_i_edit_repos()?;

    let (storage, repository) = path.into_inner();
    let vec = b.to_vec();
    let string = String::from_utf8(vec);
    if let Err(error) = string {
        error!("Unable to Parse String from Request: {}", error);
        return bad_request("Bad Description");
    }

    let storage = storages.get_storage_by_name(&storage).await?;
    if storage.is_none() {
        return not_found();
    }
    let storage = storage.unwrap();
    let repository = storage.get_repository(&repository).await?;
    if repository.is_none() {
        return not_found();
    }
    let mut repository = repository.unwrap();
    repository.settings.description = string.unwrap();
    storage.update_repository(&repository).await?;
    APIResponse::new(true, Some(repository)).respond(&r)
}

#[patch("/api/admin/repository/{storage}/{repository}/modify/settings/frontend")]
pub async fn modify_frontend_settings(
    connection: web::Data<DatabaseConnection>, storages: web::Data<StorageManager>,
    site: NitroRepoData,
    r: HttpRequest, auth: Authentication,
    path: web::Path<(String, String)>,
    nc: web::Json<Frontend>,
) -> SiteResponse {
    let caller: UserModel = auth.get_user(&connection).await??;
    caller.can_i_edit_repos()?;
    let (storage, repository) = path.into_inner();

    let storage = storages.get_storage_by_name(&storage).await?;
    if storage.is_none() {
        return not_found();
    }
    let storage = storage.unwrap();
    let repository = storage.get_repository(&repository).await?;
    if repository.is_none() {
        return not_found();
    }
    let mut repository = repository.unwrap();
    repository.settings.frontend = nc.0;
    storage.update_repository(&repository).await?;
    APIResponse::new(true, Some(repository)).respond(&r)
}

#[patch("/api/admin/repository/{storage}/{repository}/modify/settings/badge")]
pub async fn modify_badge_settings(
    connection: web::Data<DatabaseConnection>, storages: web::Data<StorageManager>,
    site: NitroRepoData,
    r: HttpRequest, auth: Authentication,
    path: web::Path<(String, String)>,
    nc: web::Json<BadgeSettings>,
) -> SiteResponse {
    let caller: UserModel = auth.get_user(&connection).await??;
    caller.can_i_edit_repos()?;
    let (storage, repository) = path.into_inner();
    let storage = storages.get_storage_by_name(&storage).await?;
    if storage.is_none() {
        return not_found();
    }
    let storage = storage.unwrap();
    let repository = storage.get_repository(&repository).await?;
    if repository.is_none() {
        return not_found();
    }
    let mut repository = repository.unwrap();
    repository.settings.badge = nc.0;
    storage.update_repository(&repository).await?;
    APIResponse::new(true, Some(repository)).respond(&r)
}

#[patch("/api/admin/repository/{storage}/{repository}/modify/security/visibility/{visibility}")]
pub async fn modify_security(
    connection: web::Data<DatabaseConnection>,
    site: NitroRepoData, storages: web::Data<StorageManager>,
    r: HttpRequest, auth: Authentication,
    path: web::Path<(String, String, Visibility)>,
) -> SiteResponse {
    let caller: UserModel = auth.get_user(&connection).await??;
    caller.can_i_edit_repos()?;
    let (storage, repository, visibility) = path.into_inner();

    let storage = storages.get_storage_by_name(&storage).await?;
    if storage.is_none() {
        return not_found();
    }
    let storage = storage.unwrap();
    let repository = storage.get_repository(&repository).await?;
    if repository.is_none() {
        return not_found();
    }
    let mut repository = repository.unwrap();
    repository.security.visibility = visibility;
    storage.update_repository(&repository).await?;
    APIResponse::new(true, Some(repository)).respond(&r)
}

#[patch("/api/admin/repository/{storage}/{repository}/modify/deploy/report")]
pub async fn modify_deploy(
    connection: web::Data<DatabaseConnection>,
    site: NitroRepoData,
    r: HttpRequest, auth: Authentication, storages: web::Data<StorageManager>,
    path: web::Path<(String, String)>,
    nc: web::Json<ReportGeneration>,
) -> SiteResponse {
    let caller: UserModel = auth.get_user(&connection).await??;
    caller.can_i_edit_repos()?;
    let (storage, repository) = path.into_inner();
    let storage = storages.get_storage_by_name(&storage).await?;

    if storage.is_none() {
        return not_found();
    }
    let storage = storage.unwrap();
    let repository = storage.get_repository(&repository).await?;
    if repository.is_none() {
        return not_found();
    }
    let mut repository = repository.unwrap();
    repository.deploy_settings.report_generation = nc.0;
    storage.update_repository(&repository).await?;

    APIResponse::respond_new(Some(repository), &r)
}

#[put("/api/admin/repository/{storage}/{repository}/modify/deploy/webhook/add")]
pub async fn add_webhook(
    connection: web::Data<DatabaseConnection>, storages: web::Data<StorageManager>,
    site: NitroRepoData,
    r: HttpRequest, auth: Authentication,
    path: web::Path<(String, String)>,
    nc: web::Json<Webhook>,
) -> SiteResponse {
    let caller: UserModel = auth.get_user(&connection).await??;
    caller.can_i_edit_repos()?;
    let (storage, repository) = path.into_inner();

    let storage = storages.get_storage_by_name(&storage).await?;

    if storage.is_none() {
        return not_found();
    }
    let storage = storage.unwrap();
    let repository = storage.get_repository(&repository).await?;
    if repository.is_none() {
        return not_found();
    }
    let mut repository = repository.unwrap();
    repository.deploy_settings.add_webhook(nc.0);
    storage.update_repository(&repository).await?;
    APIResponse::respond_new(Some(repository), &r)
}

#[delete("/api/admin/repository/{storage}/{repository}/modify/deploy/webhook/{webhook}")]
pub async fn remove_webhook(
    connection: web::Data<DatabaseConnection>,
    site: NitroRepoData, storages: web::Data<StorageManager>,
    r: HttpRequest, auth: Authentication,
    path: web::Path<(String, String, String)>,
) -> SiteResponse {
    let caller: UserModel = auth.get_user(&connection).await??;
    caller.can_i_edit_repos()?;
    let (storage, repository, webhook) = path.into_inner();

    let storage = storages.get_storage_by_name(&storage).await?;

    if storage.is_none() {
        return not_found();
    }
    let storage = storage.unwrap();
    let repository = storage.get_repository(&repository).await?;
    if repository.is_none() {
        return not_found();
    }
    let mut repository = repository.unwrap();
    repository.deploy_settings.remove_hook(webhook);
    storage.update_repository(&repository).await?;
    APIResponse::respond_new(Some(repository), &r)
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DeleteRequest {
    pub delete_files: Option<bool>,
}

#[delete("/api/admin/repository/{storage}/{repository}")]
pub async fn delete_repository(
    connection: web::Data<DatabaseConnection>,
    site: NitroRepoData,
    r: HttpRequest, auth: Authentication,
    path: web::Path<(String, String)>, storages: web::Data<StorageManager>,
    query: web::Query<DeleteRequest>,
) -> SiteResponse {
    let caller: UserModel = auth.get_user(&connection).await??;
    caller.can_i_edit_repos()?;
    let (storage, repository) = path.into_inner();

    let storage = storages.get_storage_by_name(&storage).await?;

    if storage.is_none() {
        return not_found();
    }
    let storage = storage.unwrap();
    let repository = storage.get_repository(&repository).await?;
    if repository.is_none() {
        return not_found();
    }
    let repository = repository.unwrap();
    storage.delete_repository(&repository, query.delete_files.unwrap_or(false)).await?;
    APIResponse::from(true).respond(&r)
}

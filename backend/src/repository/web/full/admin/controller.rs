use actix_web::web::Path;
use actix_web::{delete, get, patch, post, web, HttpRequest};

use sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::api_response::{APIResponse, SiteResponse};
use crate::authentication::Authentication;

use crate::error::response::not_found;

use crate::repository::settings::security::Visibility;

use crate::repository::data::{RepositoryType, RepositoryValue};
use crate::repository::settings::Policy;
use crate::storage::multi::MultiStorageController;
use crate::system::permissions::options::CanIDo;
use crate::system::user::UserModel;
use crate::NitroRepoData;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListRepositories {
    pub repositories: Vec<RepositoryValue>,
}

#[get("/api/admin/repositories/{storage}/list")]
pub async fn list_repos_by_storage(
    connection: web::Data<DatabaseConnection>,
    _site: NitroRepoData,
    r: HttpRequest,
    auth: Authentication,
    storages: web::Data<MultiStorageController>,
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
    let vec = storage.get_repositories().await?;

    let response = ListRepositories { repositories: vec };
    APIResponse::new(true, Some(response)).respond(&r)
}

#[get("/api/admin/repositories/get/{storage}/{repo}")]
pub async fn get_repo(
    connection: web::Data<DatabaseConnection>,
    _site: NitroRepoData,
    r: HttpRequest,
    auth: Authentication,
    storages: web::Data<MultiStorageController>,
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
    let repository = storage.get_repository::<Value>(&repo).await?;

    APIResponse::new(true, repository).respond(&r)
}

#[derive(serde::Deserialize, Debug, Clone)]
pub struct NewRepository {
    pub storage: String,
    pub name: String,
    pub repo_type: RepositoryType,
}

#[post("/api/admin/repository/add")]
pub async fn add_repo(
    connection: web::Data<DatabaseConnection>,
    _site: NitroRepoData,
    auth: Authentication,
    storages: web::Data<MultiStorageController>,
    r: HttpRequest,
    nc: web::Json<NewRepository>,
) -> SiteResponse {
    let caller: UserModel = auth.get_user(&connection).await??;
    caller.can_i_edit_repos()?;
    let storage = storages.get_storage_by_name(&nc.storage).await?;
    if storage.is_none() {
        return not_found();
    }
    //TODO Verify supported repository type
    let storage = storage.unwrap();
    let repository = storage.create_repository(nc.0.name, nc.0.repo_type).await?;

    APIResponse::from(Some(repository)).respond(&r)
}

#[patch("/api/admin/repository/{storage}/{repository}/active/{active}")]
pub async fn update_active_status(
    connection: web::Data<DatabaseConnection>,
    _site: NitroRepoData,
    r: HttpRequest,
    auth: Authentication,
    storages: web::Data<MultiStorageController>,
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
    let repository = storage.get_repository::<Value>(&repo).await?;
    if repository.is_none() {
        return not_found();
    }
    let repository = repository.unwrap();
    let mut main_config = repository.main_config;
    main_config.active = active;
    main_config.update(&storage).await?;
    APIResponse::new(true, Some(true)).respond(&r)
}

#[patch("/api/admin/repository/{storage}/{repository}/policy/{policy}")]
pub async fn update_policy(
    connection: web::Data<DatabaseConnection>,
    _site: NitroRepoData,
    r: HttpRequest,
    auth: Authentication,
    storages: web::Data<MultiStorageController>,
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
    let repository = storage.get_repository::<Value>(&repository).await?;
    if repository.is_none() {
        return not_found();
    }
    let repository = repository.unwrap();
    let mut main_config = repository.main_config;
    main_config.policy = policy;
    main_config.update(&storage).await?;
    APIResponse::new(true, Some(true)).respond(&r)
}

#[patch("/api/admin/repository/{storage}/{repository}/modify/security/visibility/{visibility}")]
pub async fn modify_security(
    connection: web::Data<DatabaseConnection>,
    _site: NitroRepoData,
    storages: web::Data<MultiStorageController>,
    r: HttpRequest,
    auth: Authentication,
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
    let repository = storage.get_repository::<Value>(&repository).await?;
    if repository.is_none() {
        return not_found();
    }
    let repository = repository.unwrap();
    let mut main_config = repository.main_config;
    main_config.security.visibility = visibility;
    main_config.update(&storage).await?;
    APIResponse::new(true, Some(true)).respond(&r)
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DeleteRequest {
    pub delete_files: Option<bool>,
}

#[delete("/api/admin/repository/{storage}/{repository}")]
pub async fn delete_repository(
    connection: web::Data<DatabaseConnection>,
    _site: NitroRepoData,
    r: HttpRequest,
    auth: Authentication,
    path: web::Path<(String, String)>,
    storages: web::Data<MultiStorageController>,
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
    let repository = storage.get_repository::<Value>(&repository).await?;
    if repository.is_none() {
        return not_found();
    }
    let repository = repository.unwrap();
    storage
        .delete_repository(&repository, query.delete_files.unwrap_or(false))
        .await?;
    APIResponse::from(true).respond(&r)
}

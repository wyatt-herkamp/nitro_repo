use actix_web::web::{Bytes, Path};
use actix_web::{delete, get, patch, post, put, web, HttpRequest};
use log::error;
use sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};

use crate::api_response::{APIResponse, SiteResponse};
use crate::constants::SUPPORTED_REPO_TYPES;
use crate::error::response::{bad_request, not_found, unauthorized};
use crate::NitroRepoData;
use crate::repository::models::RepositorySummary;
use crate::repository::settings::frontend::{BadgeSettings, Frontend};
use crate::repository::settings::Policy;
use crate::repository::settings::security::Visibility;
use crate::repository::settings::webhook::{ReportGeneration, Webhook};
use crate::system::permissions::options::CanIDo;

use crate::system::utils::get_user_by_header;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListRepositories {
    pub repositories: Vec<RepositorySummary>,
}

///    let user = match user.can_i_edit_repos() {
//         Ok(user) => {
//             user
//         }
//         Err(_) => return unauthorized()
//     };
#[get("/api/admin/repositories/list")]
pub async fn list_repos(
    connection: web::Data<DatabaseConnection>,
    site: NitroRepoData,
    r: HttpRequest,
) -> SiteResponse {
    let user = get_user_by_header(r.headers(), &connection).await?;
    if user.can_i_edit_repos().is_err() {
        return unauthorized();
    }
    //TODO Change the frontend to only show repos based on the current storage being looked at.
    let mut vec = Vec::new();
    let result = site.storages.read().await;
    for (_, storage) in result.iter() {
        for (_, repo) in storage.get_repositories()? {
            vec.push(repo);
        }
    }
    let response = ListRepositories { repositories: vec };
    APIResponse::new(true, Some(response)).respond(&r)
}

#[get("/api/admin/repositories/{storage}/list")]
pub async fn list_repos_by_storage(
    connection: web::Data<DatabaseConnection>,
    site: NitroRepoData,
    r: HttpRequest,
    storage: Path<String>,
) -> SiteResponse {


    let user = get_user_by_header(r.headers(), &connection).await?;
    if user.can_i_edit_repos().is_err() {
        return unauthorized();
    }
    let result = site.storages.read().await;
    let storage = storage.into_inner();
    let storage = result.get(&storage);
    if storage.is_none() {
        return not_found();
    }
    let storage = storage.unwrap();
    let mut vec = Vec::new();
    for (_, repo) in storage.get_repositories()? {
        vec.push(repo);
    }
    let response = ListRepositories { repositories: vec };
    APIResponse::new(true, Some(response)).respond(&r)
}

#[get("/api/admin/repositories/get/{storage}/{repo}")]
pub async fn get_repo(
    connection: web::Data<DatabaseConnection>,
    site: NitroRepoData,
    r: HttpRequest,
    path: web::Path<(String, String)>,
) -> SiteResponse {
    let (storage, repo) = path.into_inner();


    if get_user_by_header(r.headers(), &connection).await?.can_i_edit_repos().is_err() {
        return unauthorized();
    }
    let result = site.storages.read().await;
    let storage = result.get(&storage);
    if storage.is_none() {
        return not_found();
    }
    let storage = storage.unwrap();
    let repository = storage.get_repository(&repo)?;
    APIResponse::from(repository).respond(&r)
}

#[post("/api/admin/repository/add")]
pub async fn add_repo(
    connection: web::Data<DatabaseConnection>,
    site: NitroRepoData,
    r: HttpRequest,
    nc: web::Json<RepositorySummary>,
) -> SiteResponse {


    if get_user_by_header(r.headers(), &connection).await?.can_i_edit_repos().is_err() {
        return unauthorized();
    }
    let result = site.storages.read().await;
    let storage = result.get(&nc.storage);
    if storage.is_none() {
        return not_found();
    }
    if !SUPPORTED_REPO_TYPES.contains(&nc.repo_type.as_str()) {
        return bad_request(format!("Unsupported Repository Type {}", &nc.repo_type));
    }
    let storage = storage.unwrap();
    let repository = storage.create_repository(nc.0)?;

    APIResponse::from(Some(repository)).respond(&r)
}

#[patch("/api/admin/repository/{storage}/{repository}/active/{active}")]
pub async fn update_active_status(
    connection: web::Data<DatabaseConnection>,
    site: NitroRepoData,
    r: HttpRequest,
    path: web::Path<(String, String, bool)>,
) -> SiteResponse {

    if get_user_by_header(r.headers(), &connection).await?.can_i_edit_repos().is_err() {
        return unauthorized();
    }

    let (storage, repo, active) = path.into_inner();
    let result = site.storages.read().await;
    let storage = result.get(&storage);
    if storage.is_none() {
        return not_found();
    }
    let storage = storage.unwrap();
    let repository = storage.get_repository(&repo)?;
    if repository.is_none() {
        return not_found();
    }
    let mut repository = repository.unwrap();
    repository.settings.active = active;
    storage.update_repository(&repository)?;
    APIResponse::new(true, Some(repository)).respond(&r)
}

#[patch("/api/admin/repository/{storage}/{repository}/policy/{policy}")]
pub async fn update_policy(
    connection: web::Data<DatabaseConnection>,
    site: NitroRepoData,
    r: HttpRequest,
    path: web::Path<(String, String, Policy)>,
) -> SiteResponse {

    if get_user_by_header(r.headers(), &connection).await?.can_i_edit_repos().is_err() {
        return unauthorized();
    }

    let (storage, repository, policy) = path.into_inner();
    let result = site.storages.read().await;
    let storage = result.get(&storage);
    if storage.is_none() {
        return not_found();
    }
    let storage = storage.unwrap();
    let repository = storage.get_repository(&repository)?;
    if repository.is_none() {
        return not_found();
    }
    let mut repository = repository.unwrap();
    repository.settings.policy = policy;
    storage.update_repository(&repository)?;
    APIResponse::new(true, Some(repository)).respond(&r)
}

#[patch("/api/admin/repository/{storage}/{repository}/description")]
pub async fn update_description(
    connection: web::Data<DatabaseConnection>,
    site: NitroRepoData,
    r: HttpRequest,
    b: Bytes,
    path: web::Path<(String, String)>,
) -> SiteResponse {

    if get_user_by_header(r.headers(), &connection).await?.can_i_edit_repos().is_err() {
        return unauthorized();
    }

    let (storage, repository) = path.into_inner();
    let vec = b.to_vec();
    let string = String::from_utf8(vec);
    if let Err(error) = string {
        error!("Unable to Parse String from Request: {}", error);
        return bad_request("Bad Description");
    }

    let result = site.storages.read().await;
    let storage = result.get(&storage);
    if storage.is_none() {
        return not_found();
    }
    let storage = storage.unwrap();
    let repository = storage.get_repository(&repository)?;
    if repository.is_none() {
        return not_found();
    }
    let mut repository = repository.unwrap();
    repository.settings.description = string.unwrap();
    storage.update_repository(&repository)?;
    APIResponse::new(true, Some(repository)).respond(&r)
}

#[patch("/api/admin/repository/{storage}/{repository}/modify/settings/frontend")]
pub async fn modify_frontend_settings(
    connection: web::Data<DatabaseConnection>,
    site: NitroRepoData,
    r: HttpRequest,
    path: web::Path<(String, String)>,
    nc: web::Json<Frontend>,
) -> SiteResponse {

    if get_user_by_header(r.headers(), &connection).await?.can_i_edit_repos().is_err() {
        return unauthorized();
    }
    let (storage, repository) = path.into_inner();

    let result = site.storages.read().await;
    let storage = result.get(&storage);
    if storage.is_none() {
        return not_found();
    }
    let storage = storage.unwrap();
    let repository = storage.get_repository(&repository)?;
    if repository.is_none() {
        return not_found();
    }
    let mut repository = repository.unwrap();
    repository.settings.frontend = nc.0;
    storage.update_repository(&repository)?;
    APIResponse::new(true, Some(repository)).respond(&r)
}

#[patch("/api/admin/repository/{storage}/{repository}/modify/settings/badge")]
pub async fn modify_badge_settings(
    connection: web::Data<DatabaseConnection>,
    site: NitroRepoData,
    r: HttpRequest,
    path: web::Path<(String, String)>,
    nc: web::Json<BadgeSettings>,
) -> SiteResponse {

    if get_user_by_header(r.headers(), &connection).await?.can_i_edit_repos().is_err() {
        return unauthorized();
    }
    let (storage, repository) = path.into_inner();
    let result = site.storages.read().await;
    let storage = result.get(&storage);
    if storage.is_none() {
        return not_found();
    }
    let storage = storage.unwrap();
    let repository = storage.get_repository(&repository)?;
    if repository.is_none() {
        return not_found();
    }
    let mut repository = repository.unwrap();
    repository.settings.badge = nc.0;
    storage.update_repository(&repository)?;
    APIResponse::new(true, Some(repository)).respond(&r)
}

#[patch("/api/admin/repository/{storage}/{repository}/modify/security/visibility/{visibility}")]
pub async fn modify_security(
    connection: web::Data<DatabaseConnection>,
    site: NitroRepoData,
    r: HttpRequest,
    path: web::Path<(String, String, Visibility)>,
) -> SiteResponse {


    if get_user_by_header(r.headers(), &connection).await?.can_i_edit_repos().is_err() {
        return unauthorized();
    }
    let (storage, repository, visibility) = path.into_inner();

    let result = site.storages.read().await;
    let storage = result.get(&storage);
    if storage.is_none() {
        return not_found();
    }
    let storage = storage.unwrap();
    let repository = storage.get_repository(&repository)?;
    if repository.is_none() {
        return not_found();
    }
    let mut repository = repository.unwrap();
    repository.security.visibility = visibility;
    storage.update_repository(&repository)?;
    APIResponse::new(true, Some(repository)).respond(&r)
}


#[patch("/api/admin/repository/{storage}/{repository}/modify/deploy/report")]
pub async fn modify_deploy(
    connection: web::Data<DatabaseConnection>,
    site: NitroRepoData,
    r: HttpRequest,
    path: web::Path<(String, String)>,
    nc: web::Json<ReportGeneration>,
) -> SiteResponse {


    if get_user_by_header(r.headers(), &connection).await?.can_i_edit_repos().is_err() {
        return unauthorized();
    }
    let (storage, repository) = path.into_inner();
    let result = site.storages.read().await;
    let storage = result.get(&storage);
    if storage.is_none() {
        return not_found();
    }
    let storage = storage.unwrap();
    let repository = storage.get_repository(&repository)?;
    if repository.is_none() {
        return not_found();
    }
    let mut repository = repository.unwrap();
    repository.deploy_settings.report_generation = nc.0;
    storage.update_repository(&repository)?;

    APIResponse::respond_new(Some(repository), &r)
}

#[put("/api/admin/repository/{storage}/{repository}/modify/deploy/webhook/add")]
pub async fn add_webhook(
    connection: web::Data<DatabaseConnection>,
    site: NitroRepoData,
    r: HttpRequest,
    path: web::Path<(String, String)>,
    nc: web::Json<Webhook>,
) -> SiteResponse {


    if get_user_by_header(r.headers(), &connection).await?.can_i_edit_repos().is_err() {
        return unauthorized();
    }
    let (storage, repository) = path.into_inner();


    let result = site.storages.read().await;
    let storage = result.get(&storage);
    if storage.is_none() {
        return not_found();
    }
    let storage = storage.unwrap();
    let repository = storage.get_repository(&repository)?;
    if repository.is_none() {
        return not_found();
    }
    let mut repository = repository.unwrap();
    repository.deploy_settings.add_webhook(nc.0);
    storage.update_repository(&repository)?;
    APIResponse::respond_new(Some(repository), &r)
}

#[delete("/api/admin/repository/{storage}/{repository}/modify/deploy/webhook/{webhook}")]
pub async fn remove_webhook(
    connection: web::Data<DatabaseConnection>,
    site: NitroRepoData,
    r: HttpRequest,
    path: web::Path<(String, String, String)>,
) -> SiteResponse {


    if get_user_by_header(r.headers(), &connection).await?.can_i_edit_repos().is_err() {
        return unauthorized();
    }
    let (storage, repository, webhook) = path.into_inner();

    let result = site.storages.read().await;
    let storage = result.get(&storage);
    if storage.is_none() {
        return not_found();
    }
    let storage = storage.unwrap();
    let repository = storage.get_repository(&repository)?;
    if repository.is_none() {
        return not_found();
    }
    let mut repository = repository.unwrap();
    repository.deploy_settings.remove_hook(webhook);
    storage.update_repository(&repository)?;
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
    r: HttpRequest,
    path: web::Path<(String, String)>,
    query: web::Query<DeleteRequest>,
) -> SiteResponse {


    if get_user_by_header(r.headers(), &connection).await?.can_i_edit_repos().is_err() {
        return unauthorized();
    }
    let (storage, repository) = path.into_inner();


    let result = site.storages.read().await;
    let storage = result.get(&storage);
    if storage.is_none() {
        return not_found();
    }
    let storage = storage.unwrap();
    let repository = storage.get_repository(&repository)?;
    if repository.is_none() {
        return not_found();
    }
    let repository = repository.unwrap();
    storage.delete_repository(&repository, query.delete_files.unwrap_or(false))?;
    APIResponse::from(true).respond(&r)
}

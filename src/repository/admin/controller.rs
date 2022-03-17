
use actix_web::web::Bytes;
use actix_web::{delete, get, patch, post, put, web, HttpRequest};
use log::error;
use serde::{Deserialize, Serialize};

use crate::api_response::{APIResponse, SiteResponse};
use crate::database::DbPool;
use crate::error::response::{ bad_request, not_found, unauthorized};
use crate::NitroRepoData;

use crate::repository::models::{BadgeSettings, Frontend, Policy, RepositorySummary, Visibility};
use crate::repository::models::{ReportGeneration, Webhook};
use crate::system::action::get_user_by_username;
use crate::system::utils::get_user_by_header;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListRepositories {
    pub repositories: Vec<RepositorySummary>,
}

#[get("/api/admin/repositories/list")]
pub async fn list_repos(pool: web::Data<DbPool>, site: NitroRepoData, r: HttpRequest) -> SiteResponse {
    let connection = pool.get()?;

    let user = get_user_by_header(r.headers(), &connection)?;
    if user.is_none() || !user.unwrap().permissions.admin {
        return unauthorized();
    }
    //TODO Change the frontend to only show repos based on the current storage being looked at.
    let mut vec = Vec::new();
    let result = site.storages.lock().unwrap();
    for (_, storage) in result.iter() {
        for (_, repo) in storage.get_repositories()? {
            vec.push(repo);
        }
    }
    let response = ListRepositories { repositories: vec };
    APIResponse::new(true, Some(response)).respond(&r)
}


#[get("/api/admin/repositories/get/{storage}/{repo}")]
pub async fn get_repo(
    pool: web::Data<DbPool>,
    site: NitroRepoData,
    r: HttpRequest,
    path: web::Path<(String, String)>,
) -> SiteResponse {
    let (storage, repo) = path.into_inner();
    let connection = pool.get()?;

    let user = get_user_by_header(r.headers(), &connection)?;
    if user.is_none() {
        return unauthorized();
    }
    let user = user.unwrap();
    if !user.permissions.deployer && !user.permissions.admin {
        return unauthorized();
    }
    let result = site.storages.lock().unwrap();
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
    pool: web::Data<DbPool>, site: NitroRepoData,
    r: HttpRequest,
    nc: web::Json<RepositorySummary>,
) -> SiteResponse {
    let connection = pool.get()?;

    let user = get_user_by_header(r.headers(), &connection)?;
    if user.is_none() || !user.unwrap().permissions.admin {
        return unauthorized();
    }
    let result = site.storages.lock().unwrap();
    let storage = result.get(&nc.storage);
    if storage.is_none() {
        return not_found();
    }
    let storage = storage.unwrap();
    let repository = storage.create_repository(nc.0)?;

    APIResponse::from(Some(repository)).respond(&r)
}

#[patch("/api/admin/repository/{storage}/{repository}/active/{active}")]
pub async fn update_active_status(
    pool: web::Data<DbPool>, site: NitroRepoData,
    r: HttpRequest,
    path: web::Path<(String, String, bool)>,
) -> SiteResponse {
    let (storage, repo, active) = path.into_inner();

    let connection = pool.get()?;

    let user = get_user_by_header(r.headers(), &connection)?;
    if user.is_none() || !user.unwrap().permissions.admin {
        return unauthorized();
    }
    let result = site.storages.lock().unwrap();
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
    pool: web::Data<DbPool>, site: NitroRepoData,
    r: HttpRequest,
    path: web::Path<(String, String, Policy)>,
) -> SiteResponse {
    let (storage, repository, policy) = path.into_inner();

    let connection = pool.get()?;

    let user = get_user_by_header(r.headers(), &connection)?;
    if user.is_none() || !user.unwrap().permissions.admin {
        return unauthorized();
    }
    let result = site.storages.lock().unwrap();
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
    pool: web::Data<DbPool>, site: NitroRepoData,
    r: HttpRequest,
    b: Bytes,
    path: web::Path<(String, String)>,
) -> SiteResponse {
    let (storage, repository) = path.into_inner();

    let connection = pool.get()?;

    let user = get_user_by_header(r.headers(), &connection)?;
    if user.is_none() || !user.unwrap().permissions.admin {
        return unauthorized();
    }

    let vec = b.to_vec();
    let string = String::from_utf8(vec);
    if let Err(error) = string {
        error!("Unable to Parse String from Request: {}", error);
        return bad_request("Bad Description");
    }


    let result = site.storages.lock().unwrap();
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
    pool: web::Data<DbPool>, site: NitroRepoData,
    r: HttpRequest,
    path: web::Path<(String, String)>,
    nc: web::Json<Frontend>,
) -> SiteResponse {
    let (storage, repository) = path.into_inner();
    let connection = pool.get()?;

    let user = get_user_by_header(r.headers(), &connection)?;
    if user.is_none() || !user.unwrap().permissions.admin {
        return unauthorized();
    }
    let result = site.storages.lock().unwrap();
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
    pool: web::Data<DbPool>, site: NitroRepoData,
    r: HttpRequest,
    path: web::Path<(String, String)>,
    nc: web::Json<BadgeSettings>,
) -> SiteResponse {
    let (storage, repository) = path.into_inner();

    let connection = pool.get()?;

    let user = get_user_by_header(r.headers(), &connection)?;
    if user.is_none() || !user.unwrap().permissions.admin {
        return unauthorized();
    }
    let result = site.storages.lock().unwrap();
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
    pool: web::Data<DbPool>, site: NitroRepoData,
    r: HttpRequest,
    path: web::Path<(String, String, Visibility)>,
) -> SiteResponse {
    let (storage, repository, visibility) = path.into_inner();
    println!("{:?}", &visibility);
    let connection = pool.get()?;

    let user = get_user_by_header(r.headers(), &connection)?;
    if user.is_none() || !user.unwrap().permissions.admin {
        return unauthorized();
    }
    let result = site.storages.lock().unwrap();
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

#[patch("/api/admin/repository/{storage}/{repository}/clear/security/{what}")]
pub async fn clear_all(
    pool: web::Data<DbPool>, site: NitroRepoData,
    r: HttpRequest,
    path: web::Path<(String, String, String)>,
) -> SiteResponse {
    let (storage, repository, what) = path.into_inner();

    let connection = pool.get()?;

    let admin = get_user_by_header(r.headers(), &connection)?;
    if admin.is_none() || !admin.unwrap().permissions.admin {
        return unauthorized();
    }
    let result = site.storages.lock().unwrap();
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

    match what.as_str() {
        "deployers" => repository.security.deployers.clear(),
        "readers" => repository.security.readers.clear(),
        _ => return bad_request("Must be Deployers or Readers"),
    }
    storage.update_repository(&repository)?;
    APIResponse::new(true, Some(repository)).respond(&r)
}

#[patch("/api/admin/repository/{storage}/{repository}/modify/security/{what}/{action}/{user}")]
pub async fn update_deployers_readers(
    pool: web::Data<DbPool>, site: NitroRepoData,
    r: HttpRequest,
    path: web::Path<(String, String, String, String, String)>,
) -> SiteResponse {
    let (storage, repository, what, action, user) = path.into_inner();

    let connection = pool.get()?;

    let admin = get_user_by_header(r.headers(), &connection)?;
    if admin.is_none() || !admin.unwrap().permissions.admin {
        return unauthorized();
    }
    let result = site.storages.lock().unwrap();
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
    let user = get_user_by_username(&user, &connection)?;
    if user.is_none() {
        return not_found();
    }
    let user = user.unwrap();
    match action.as_str() {
        "deployers" => match what.as_str() {
            "add" => {
                repository.security.deployers.push(user.id);
            }
            "remove" => {
                let filter = repository
                    .security
                    .deployers
                    .iter()
                    .position(|x| x == &user.id);
                if filter.is_some() {
                    repository.security.deployers.remove(filter.unwrap());
                }
            }
            _ => return bad_request("Must be Add or Remove"),
        },
        "readers" => match what.as_str() {
            "add" => {
                repository.security.readers.push(user.id);
            }
            "remove" => {
                let filter = repository
                    .security
                    .readers
                    .iter()
                    .position(|x| x == &user.id);
                if filter.is_some() {
                    repository.security.readers.remove(filter.unwrap());
                }
            }
            _ => return bad_request("Must be Add or Remove"),
        },
        _ => return bad_request("Must be Deployers or Readers"),
    }
    storage.update_repository(&repository)?;
    APIResponse::new(true, Some(repository)).respond(&r)
}

#[patch("/api/admin/repository/{storage}/{repository}/modify/deploy/report")]
pub async fn modify_deploy(
    pool: web::Data<DbPool>, site: NitroRepoData,
    r: HttpRequest,
    path: web::Path<(String, String)>,
    nc: web::Json<ReportGeneration>,
) -> SiteResponse {
    let (storage, repository) = path.into_inner();

    let connection = pool.get()?;

    let user = get_user_by_header(r.headers(), &connection)?;
    if user.is_none() || !user.unwrap().permissions.admin {
        return unauthorized();
    }
    let result = site.storages.lock().unwrap();
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
    pool: web::Data<DbPool>, site: NitroRepoData,
    r: HttpRequest,
    path: web::Path<(String, String)>,
    nc: web::Json<Webhook>,
) -> SiteResponse {
    let (storage, repository) = path.into_inner();

    let connection = pool.get()?;

    let user = get_user_by_header(r.headers(), &connection)?;
    if user.is_none() || !user.unwrap().permissions.admin {
        return unauthorized();
    }
    let result = site.storages.lock().unwrap();
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
    pool: web::Data<DbPool>, site: NitroRepoData,
    r: HttpRequest,
    path: web::Path<(String, String, String)>,
) -> SiteResponse {
    let (storage, repository, webhook) = path.into_inner();

    let connection = pool.get()?;
    let user = get_user_by_header(r.headers(), &connection)?;
    if user.is_none() || !user.unwrap().permissions.admin {
        return unauthorized();
    }
    let result = site.storages.lock().unwrap();
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

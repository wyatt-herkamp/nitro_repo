use std::fs::create_dir_all;
use std::path::PathBuf;
use std::str::FromStr;

use actix_web::{get, HttpRequest, post, patch, web};
use serde::{Deserialize, Serialize};

use crate::api_response::{APIResponse, SiteResponse};
use crate::DbPool;
use crate::error::response::{already_exists, bad_request, not_found, unauthorized};
use crate::repository::action::{add_new_repository, get_repo_by_id, get_repo_by_name_and_storage, get_repositories, update_repo, update_repo_security, update_repo_settings};
use crate::repository::models::{BadgeSettings, Frontend, Policy, Repository, RepositoryListResponse, RepositorySettings, SecurityRules, UpdateFrontend, UpdateSettings, Visibility};
use crate::storage::action::get_storage_by_name;
use crate::system::action::get_user_by_username;
use crate::system::utils::get_user_by_header;
use crate::utils::get_current_time;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListRepositories {
    pub repositories: Vec<RepositoryListResponse>,
}

#[get("/api/repositories/list")]
pub async fn list_repos(pool: web::Data<DbPool>, r: HttpRequest) -> SiteResponse {
    let connection = pool.get()?;

    let user = get_user_by_header(r.headers(), &connection)?;
    if user.is_none() || !user.unwrap().permissions.admin {
        return unauthorized();
    }
    let vec = get_repositories(&connection)?;

    let response = ListRepositories { repositories: vec };
    APIResponse::new(true, Some(response)).respond(&r)
}

#[get("/api/repositories/get/{repo}")]
pub async fn get_repo(
    pool: web::Data<DbPool>,
    r: HttpRequest,
    path: web::Path<i64>,
) -> SiteResponse {
    let connection = pool.get()?;

    let user = get_user_by_header(r.headers(), &connection)?;
    if user.is_none() || !user.unwrap().permissions.admin {
        return unauthorized();
    }
    let repo = get_repo_by_id(&path.into_inner(), &connection)?;

    APIResponse::respond_new(repo, &r)
}

#[get("/api/repositories/get/{storage}/{repo}")]
pub async fn get_repo_deployer(
    pool: web::Data<DbPool>,
    r: HttpRequest,
    path: web::Path<(String, String)>,
) -> SiteResponse {
    let (storage, repo) = path.into_inner();
    let connection = pool.get()?;

    let user = get_user_by_header(r.headers(), &connection)?;
    if user.is_none() || !user.unwrap().permissions.deployer {
        return unauthorized();
    }
    let storage = get_storage_by_name(&storage, &connection)?;
    if storage.is_none() {
        return not_found();
    }
    let repo = get_repo_by_name_and_storage(&repo, &storage.unwrap().id, &connection)?;

    APIResponse::respond_new(repo, &r)
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct NewRepo {
    pub name: String,
    pub storage: String,
    pub repo: String,
}

#[post("/api/admin/repository/add")]
pub async fn add_repo(
    pool: web::Data<DbPool>,
    r: HttpRequest,
    nc: web::Json<NewRepo>,
) -> SiteResponse {
    let connection = pool.get()?;

    let user = get_user_by_header(r.headers(), &connection)?;
    if user.is_none() || !user.unwrap().permissions.admin {
        return unauthorized();
    }
    let storage = crate::storage::action::get_storage_by_name(&nc.storage, &connection)?;
    if storage.is_none() {
        return not_found();
    }
    let storage = storage.unwrap();

    let option = get_repo_by_name_and_storage(&nc.name, &storage.id, &connection)?;
    if option.is_some() {
        return already_exists();
    }
    let repository = Repository {
        id: 0,

        name: nc.0.name,
        repo_type: nc.0.repo,
        storage: storage.id,
        settings: RepositorySettings::default(),
        security: SecurityRules {
            deployers: vec![],
            visibility: Visibility::Public,
            readers: vec![],
        },
        created: get_current_time(),
    };
    add_new_repository(&repository, &connection)?;
    let buf = PathBuf::new()
        .join("storages")
        .join(&storage.name)
        .join(&repository.name);
    if !buf.exists() {
        create_dir_all(buf)?;
    }
    let option = get_repo_by_name_and_storage(&repository.name, &storage.id, &connection)?;

    APIResponse::from(option).respond(&r)
}

#[patch("/api/admin/repository/{repo}/active/{active}")]
pub async fn update_active_status(
    pool: web::Data<DbPool>,
    r: HttpRequest,
    path: web::Path<(i64, bool)>,
) -> SiteResponse {
    let (repo, active) = path.into_inner();

    let connection = pool.get()?;

    let user = get_user_by_header(r.headers(), &connection)?;
    if user.is_none() || !user.unwrap().permissions.admin {
        return unauthorized();
    }
    let repository = get_repo_by_id(&repo, &connection)?;

    let mut repository = repository.unwrap();
    repository.settings.active = active;
    update_repo_settings(&repo, &repository.settings, &connection)?;
    APIResponse::new(true, Some(repository)).respond(&r)
}

#[patch("/api/admin/repository/{repo}/policy/{policy}")]
pub async fn update_policy(
    pool: web::Data<DbPool>,
    r: HttpRequest,
    path: web::Path<(i64, Policy)>,
) -> SiteResponse {
    let (repo, policy) = path.into_inner();

    let connection = pool.get()?;

    let user = get_user_by_header(r.headers(), &connection)?;
    if user.is_none() || !user.unwrap().permissions.admin {
        return unauthorized();
    }
    let repository = get_repo_by_id(&repo, &connection)?;

    let mut repository = repository.unwrap();
    repository.settings.policy = policy;
    update_repo_settings(&repo, &repository.settings, &connection)?;
    APIResponse::new(true, Some(repository)).respond(&r)
}


#[patch("/api/admin/repository/{repo}/modify/settings/frontend")]
pub async fn modify_frontend_settings(
    pool: web::Data<DbPool>,
    r: HttpRequest,
    path: web::Path<(i64)>,
    nc: web::Json<Frontend>,
) -> SiteResponse {
    let connection = pool.get()?;

    let user = get_user_by_header(r.headers(), &connection)?;
    if user.is_none() || !user.unwrap().permissions.admin {
        return unauthorized();
    }
    let repository = get_repo_by_id(&path.into_inner(), &connection)?;

    let mut repository = repository.unwrap();
    repository.settings.frontend = nc.0;
    update_repo_settings(&repository.id, &repository.settings, &connection)?;
    APIResponse::new(true, Some(repository)).respond(&r)
}
#[patch("/api/admin/repository/{repo}/modify/settings/badge")]
pub async fn modify_badge_settings(
    pool: web::Data<DbPool>,
    r: HttpRequest,
    path: web::Path<(i64)>,
    nc: web::Json<BadgeSettings>,
) -> SiteResponse {
    let connection = pool.get()?;

    let user = get_user_by_header(r.headers(), &connection)?;
    if user.is_none() || !user.unwrap().permissions.admin {
        return unauthorized();
    }
    let repository = get_repo_by_id(&path.into_inner(), &connection)?;

    let mut repository = repository.unwrap();
    repository.settings.badge = nc.0;
    update_repo_settings(&repository.id, &repository.settings, &connection)?;
    APIResponse::new(true, Some(repository)).respond(&r)
}

#[patch("/api/admin/repository/{repo}/modify/security/visibility/{visibility}")]
pub async fn modify_security(
    pool: web::Data<DbPool>,
    r: HttpRequest,
    path: web::Path<(i64, Visibility)>,
) -> SiteResponse {
    let (repo, visibility) = path.into_inner();

    let connection = pool.get()?;

    let user = get_user_by_header(r.headers(), &connection)?;
    if user.is_none() || !user.unwrap().permissions.admin {
        return unauthorized();
    }
    let repository = get_repo_by_id(&repo, &connection)?;

    let mut repository = repository.unwrap();
    //TODO BAD CODE
    repository.security.visibility = visibility;
    update_repo_security(&repository.id, &repository.security, &connection)?;
    APIResponse::new(true, Some(repository)).respond(&r)
}

#[patch("/api/admin/repository/{repo}/clear/security/{what}")]
pub async fn clear_all(
    pool: web::Data<DbPool>,
    r: HttpRequest,
    path: web::Path<(i64, String)>,
) -> SiteResponse {
    let (repo, what) = path.into_inner();

    let connection = pool.get()?;

    let admin = get_user_by_header(r.headers(), &connection)?;
    if admin.is_none() || !admin.unwrap().permissions.admin {
        return unauthorized();
    }
    let repository = get_repo_by_id(&repo, &connection)?;

    let mut repository = repository.unwrap();

    match what.as_str() {
        "deployers" => {
            repository.security.deployers.clear()
        }
        "readers" => {
            repository.security.readers.clear()
        }
        _ => return bad_request("Must be Deployers or Readers"),
    }
    update_repo_security(&repository.id, &repository.security, &connection)?;
    APIResponse::new(true, Some(repository)).respond(&r)
}

#[patch("/api/admin/repository/{repo}/modify/security/{what}/{action}/{user}")]
pub async fn update_deployers_readers(
    pool: web::Data<DbPool>,
    r: HttpRequest,
    path: web::Path<(i64, String, String, String)>,
) -> SiteResponse {
    let (repo, what, action, user) = path.into_inner();

    let connection = pool.get()?;

    let admin = get_user_by_header(r.headers(), &connection)?;
    if admin.is_none() || !admin.unwrap().permissions.admin {
        return unauthorized();
    }
    let repository = get_repo_by_id(&repo, &connection)?;

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
    update_repo_security(&repository.id, &repository.security, &connection)?;
    APIResponse::new(true, Some(repository)).respond(&r)
}

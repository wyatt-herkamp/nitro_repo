use actix_web::{get, post, web, HttpRequest};
use serde::{Deserialize, Serialize};

use crate::api_response::APIResponse;

use crate::error::request_error::RequestError;
use crate::repository::action::{add_new_repository, get_repo_by_name_and_storage, get_repositories, update_repo};
use crate::repository::models::{Repository, RepositorySettings, SecurityRules, Visibility};
use crate::system::utils::get_user_by_header;
use crate::utils::{get_current_time, installed};
use crate::DbPool;
use std::fs::create_dir_all;
use std::path::PathBuf;
use crate::error::request_error::RequestError::{NotAuthorized, NotFound};
use crate::storage::action::get_storage_by_name;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListRepositories {
    pub repositories: Vec<Repository>,
}

#[get("/api/repositories/list")]
pub async fn list_repos(
    pool: web::Data<DbPool>,
    r: HttpRequest,
) -> Result<APIResponse<ListRepositories>, RequestError> {
    let connection = pool.get()?;
    installed(&connection)?;
    let user =
        get_user_by_header(r.headers(), &connection)?.ok_or_else(|| RequestError::NotAuthorized)?;
    if !user.permissions.admin {
        return Err(RequestError::NotAuthorized);
    }
    let vec = get_repositories(&connection)?;

    let response = ListRepositories { repositories: vec };
    return Ok(APIResponse::new(true, Some(response)));
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct NewRepo {
    pub name: String,
    pub storage: String,
    pub repo: String,
    pub settings: RepositorySettings,
}

#[post("/api/admin/repository/add")]
pub async fn add_repo(
    pool: web::Data<DbPool>,
    r: HttpRequest,
    nc: web::Json<NewRepo>,
) -> Result<APIResponse<Repository>, RequestError> {
    let connection = pool.get()?;
    installed(&connection)?;
    let user =
        get_user_by_header(r.headers(), &connection)?.ok_or_else(|| RequestError::NotAuthorized)?;
    if !user.permissions.admin {
        return Err(RequestError::NotAuthorized);
    }
    let option1 = crate::storage::action::get_storage_by_name(nc.storage.clone(), &connection)?
        .ok_or_else(|| RequestError::from("Unable to find storage"))?;

    let option = get_repo_by_name_and_storage(nc.name.clone(), option1.id, &connection)?;
    if option.is_some() {
        return Err(RequestError::AlreadyExists);
    }
    let repository = Repository {
        id: 0,

        name: nc.name.clone(),
        repo_type: nc.repo.clone(),
        storage: option1.id.clone(),
        settings: nc.settings.clone(),
        security: SecurityRules {
            open_to_all_deployers: true,
            deployers: vec![],
            visibility: Visibility::Public,
            open_to_all_readers: true,
            readers: vec![]
        },
        created: get_current_time(),
    };
    add_new_repository(&repository, &connection)?;
    let buf = PathBuf::new()
        .join("storages")
        .join(nc.name.clone())
        .join(repository.name.clone());
    if !buf.exists() {
        create_dir_all(buf)?;
    }
    let option = get_repo_by_name_and_storage(nc.name.clone(), option1.id, &connection)?
        .ok_or(RequestError::NotFound)?;
    return Ok(APIResponse::new(true, Some(option)));
}

#[post("/api/admin/repository/{storage}/{repo}/modify/settings")]
pub async fn modify_settings(
    pool: web::Data<DbPool>,
    r: HttpRequest,
    path: web::Path<(String, String)>,
    nc: web::Json<RepositorySettings>,
) -> Result<APIResponse<Repository>, RequestError> {
    let connection = pool.get()?;
    installed(&connection)?;
    let admin = get_user_by_header(r.headers(), &connection)?.ok_or_else(|| NotAuthorized)?;
    if !admin.permissions.admin {
        return Err(NotAuthorized);
    }
    let string = path.0.1.clone();
    let storage = get_storage_by_name(string, &connection)?.ok_or(NotFound)?;
    let mut repository = get_repo_by_name_and_storage(path.0.1.clone(), storage.id, &connection)?.ok_or(NotFound)?;
    repository.settings.update(nc.0.clone());
    update_repo(&repository, &connection)?;
    return Ok(APIResponse::new(true, Some(repository)));
}

#[post("/api/admin/repository/{storage}/{repo}/modify/security")]
pub async fn modify_security(
    pool: web::Data<DbPool>,
    r: HttpRequest,
    path: web::Path<(String, String)>,
    nc: web::Json<SecurityRules>,
) -> Result<APIResponse<Repository>, RequestError> {
    let connection = pool.get()?;
    installed(&connection)?;
    let admin = get_user_by_header(r.headers(), &connection)?.ok_or_else(|| NotAuthorized)?;
    if !admin.permissions.admin {
        return Err(NotAuthorized);
    }
    let string = path.0.1.clone();
    let storage = get_storage_by_name(string, &connection)?.ok_or(NotFound)?;
    let mut repository = get_repo_by_name_and_storage(path.0.1.clone(), storage.id, &connection)?.ok_or(NotFound)?;
    repository.security.update(nc.0.clone());
    update_repo(&repository, &connection)?;
    return Ok(APIResponse::new(true, Some(repository)));
}

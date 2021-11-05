use actix_web::{get, post, web, HttpRequest};
use serde::{Deserialize, Serialize};

use crate::api_response::APIResponse;

use crate::error::request_error::RequestError;
use crate::error::request_error::RequestError::{NotAuthorized, NotFound};
use crate::repository::action::{
    add_new_repository, get_repo_by_name_and_storage, get_repositories, update_repo,
};
use crate::repository::models::{Repository, RepositorySettings, SecurityRules, UpdateFrontend, UpdateSettings, Visibility};
use crate::storage::action::get_storage_by_name;
use crate::system::utils::get_user_by_header;
use crate::utils::{get_current_time, installed};
use crate::DbPool;
use std::fs::create_dir_all;
use std::path::PathBuf;
use crate::system::action::get_user_by_username;
use std::str::FromStr;
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
            deployers: vec![],
            visibility: Visibility::Public,
            readers: vec![],
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

#[post("/api/admin/repository/{storage}/{repo}/modify/settings/general")]
pub async fn modify_general_settings(
    pool: web::Data<DbPool>,
    r: HttpRequest,
    path: web::Path<(String, String)>,
    nc: web::Json<UpdateSettings>,
) -> Result<APIResponse<Repository>, RequestError> {
    let connection = pool.get()?;

    let admin = get_user_by_header(r.headers(), &connection)?.ok_or_else(|| NotAuthorized)?;
    if !admin.permissions.admin {
        return Err(NotAuthorized);
    }
    let string = path.0.0.clone();
    let storage = get_storage_by_name(string, &connection)?.ok_or(NotFound)?;
    let mut repository = get_repo_by_name_and_storage(path.0.1.clone(), storage.id, &connection)?
        .ok_or(NotFound)?;
    repository.settings.update_general(nc.0);
    update_repo(&repository, &connection)?;
    return Ok(APIResponse::new(true, Some(repository)));
}

#[post("/api/admin/repository/{storage}/{repo}/modify/settings/frontend")]
pub async fn modify_frontend_settings(
    pool: web::Data<DbPool>,
    r: HttpRequest,
    path: web::Path<(String, String)>,
    nc: web::Json<UpdateFrontend>,
) -> Result<APIResponse<Repository>, RequestError> {
    let connection = pool.get()?;

    let admin = get_user_by_header(r.headers(), &connection)?.ok_or_else(|| NotAuthorized)?;
    if !admin.permissions.admin {
        return Err(NotAuthorized);
    }
    let string = path.0.0.clone();
    let storage = get_storage_by_name(string, &connection)?.ok_or(NotFound)?;
    let mut repository = get_repo_by_name_and_storage(path.0.1.clone(), storage.id, &connection)?
        .ok_or(NotFound)?;
    repository.settings.update_frontend(nc.0);
    update_repo(&repository, &connection)?;
    return Ok(APIResponse::new(true, Some(repository)));
}

#[post("/api/admin/repository/{storage}/{repo}/modify/security/visibility/{visibility}")]
pub async fn modify_security(
    pool: web::Data<DbPool>,
    r: HttpRequest,
    path: web::Path<(String, String,String)>,
) -> Result<APIResponse<Repository>, RequestError> {
    let connection = pool.get()?;

    let admin = get_user_by_header(r.headers(), &connection)?.ok_or_else(|| NotAuthorized)?;
    if !admin.permissions.admin {
        return Err(NotAuthorized);
    }
    let string = path.0.1.clone();
    let storage = get_storage_by_name(string, &connection)?.ok_or(NotFound)?;
    let mut repository = get_repo_by_name_and_storage(path.0.1.clone(), storage.id, &connection)?
        .ok_or(NotFound)?;
    let visibility = Visibility::from_str(path.0.2.as_str()).map_err(|_e| RequestError::BadRequest("Invalid Visibility".to_string()))?;
    repository.security.set_visibility(visibility);
    update_repo(&repository, &connection)?;
    return Ok(APIResponse::new(true, Some(repository)));
}

#[post("/api/admin/repository/{storage}/{repo}/modify/security/{what}/{action}/{user}")]
pub async fn update_deployers_readers(
    pool: web::Data<DbPool>,
    r: HttpRequest,
    path: web::Path<(String, String, String, String, String)>,
) -> Result<APIResponse<Repository>, RequestError> {
    let connection = pool.get()?;

    let admin = get_user_by_header(r.headers(), &connection)?.ok_or_else(|| NotAuthorized)?;
    if !admin.permissions.admin {
        return Err(NotAuthorized);
    }
    let string = path.0.1.clone();
    let storage = get_storage_by_name(string, &connection)?.ok_or(NotFound)?;
    let mut repository = get_repo_by_name_and_storage(path.0.1.clone(), storage.id, &connection)?
        .ok_or(NotFound)?;
    let user = get_user_by_username(path.0.4, &connection)?.ok_or(NotFound)?;
    match path.0.2.as_str() {
        "deployers" => {
            match path.0.3.as_str() {
                "add" => {
                    repository.security.deployers.push(user.id);
                }
                "remove" => {
                    let filter = repository.security.deployers.iter().position(|x| x == &user.id);
                    if filter.is_some() {
                        repository.security.deployers.remove(filter.unwrap());
                    }
                }
                _ => return Err(RequestError::BadRequest("Must be Add or Remove".to_string()))
            }
        }
        "readers" => {
            match path.0.3.as_str() {
                "add" => {
                    repository.security.readers.push(user.id);
                }
                "remove" => {
                    let filter = repository.security.readers.iter().position(|x| x == &user.id);
                    if filter.is_some() {
                        repository.security.readers.remove(filter.unwrap());
                    }
                }
                _ => return Err(RequestError::BadRequest("Must be Add or Remove".to_string()))
            }
        }
        _ => return Err(RequestError::BadRequest("Must be Deployers or Readers".to_string()))
    }
    update_repo(&repository, &connection)?;
    return Ok(APIResponse::new(true, Some(repository)));
}

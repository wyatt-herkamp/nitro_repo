use actix_web::{get, post, web, HttpRequest};
use serde::{Deserialize, Serialize};

use crate::api_response::{APIResponse, SiteResponse};



use crate::repository::action::{
    add_new_repository, get_repo_by_name_and_storage, get_repositories, update_repo,
};
use crate::repository::models::{Repository, RepositorySettings, SecurityRules, UpdateFrontend, UpdateSettings, Visibility};
use crate::storage::action::get_storage_by_name;
use crate::system::utils::get_user_by_header;
use crate::utils::{get_current_time};
use crate::DbPool;
use std::fs::create_dir_all;
use std::path::PathBuf;
use crate::system::action::get_user_by_username;
use std::str::FromStr;
use crate::error::response::{already_exists, bad_request, not_found, unauthorized};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListRepositories {
    pub repositories: Vec<Repository>,
}

#[get("/api/repositories/list")]
pub async fn list_repos(
    pool: web::Data<DbPool>,
    r: HttpRequest,
) -> SiteResponse {
    let connection = pool.get()?;

    let user =
        get_user_by_header(r.headers(), &connection)?;
    if user.is_none() || !user.unwrap().permissions.admin {
        return unauthorized();
    }
    let vec = get_repositories(&connection)?;

    let response = ListRepositories { repositories: vec };
    return APIResponse::new(true, Some(response)).respond(&r);
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
) -> SiteResponse {
    let connection = pool.get()?;

    let user =
        get_user_by_header(r.headers(), &connection)?;
    if user.is_none() || !user.unwrap().permissions.admin {
        return unauthorized();
    }
    let storage = crate::storage::action::get_storage_by_name(nc.storage.clone(), &connection)?;
    if storage.is_none() {
        return not_found();
    }
    let storage = storage.unwrap();

    let option = get_repo_by_name_and_storage(nc.name.clone(), storage.id, &connection)?;
    if option.is_some() {
        return already_exists();
    }
    let repository = Repository {
        id: 0,

        name: nc.name.clone(),
        repo_type: nc.repo.clone(),
        storage: storage.id.clone(),
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
    let option = get_repo_by_name_and_storage(nc.name.clone(), storage.id, &connection)?;

    return APIResponse::from(option).respond(&r);
}

#[post("/api/admin/repository/{storage}/{repo}/modify/settings/general")]
pub async fn modify_general_settings(
    pool: web::Data<DbPool>,
    r: HttpRequest,
    path: web::Path<(String, String)>,
    nc: web::Json<UpdateSettings>,
) -> SiteResponse {
    let connection = pool.get()?;

    let user =
        get_user_by_header(r.headers(), &connection)?;
    if user.is_none() || !user.unwrap().permissions.admin {
        return unauthorized();
    }
    let string = path.0.0.clone();
    let storage = get_storage_by_name(string, &connection)?;
    if storage.is_none() {
        return not_found();
    }
    let storage = storage.unwrap();
    let repository = get_repo_by_name_and_storage(path.0.1.clone(), storage.id, &connection)?;
    if repository.is_none() {
        return not_found();
    }
    let mut repository = repository.unwrap();
    repository.settings.update_general(nc.0);
    update_repo(&repository, &connection)?;
    return APIResponse::new(true, Some(repository)).respond(&r);
}

#[post("/api/admin/repository/{storage}/{repo}/modify/settings/frontend")]
pub async fn modify_frontend_settings(
    pool: web::Data<DbPool>,
    r: HttpRequest,
    path: web::Path<(String, String)>,
    nc: web::Json<UpdateFrontend>,
) -> SiteResponse {
    let connection = pool.get()?;

    let user =
        get_user_by_header(r.headers(), &connection)?;
    if user.is_none() || !user.unwrap().permissions.admin {
        return unauthorized();
    }
    let string = path.0.0.clone();
    let storage = get_storage_by_name(string, &connection)?;
    if storage.is_none() {
        return not_found();
    }
    let storage = storage.unwrap();
    let repository = get_repo_by_name_and_storage(path.0.1.clone(), storage.id, &connection)?;
    if repository.is_none() {
        return not_found();
    }
    let mut repository = repository.unwrap();
    repository.settings.update_frontend(nc.0);
    update_repo(&repository, &connection)?;
    return APIResponse::new(true, Some(repository)).respond(&r);
}

#[post("/api/admin/repository/{storage}/{repo}/modify/security/visibility/{visibility}")]
pub async fn modify_security(
    pool: web::Data<DbPool>,
    r: HttpRequest,
    path: web::Path<(String, String, String)>,
) -> SiteResponse {
    let connection = pool.get()?;

    let user =
        get_user_by_header(r.headers(), &connection)?;
    if user.is_none() || !user.unwrap().permissions.admin {
        return unauthorized();
    }
    let string = path.0.0.clone();
    let storage = get_storage_by_name(string, &connection)?;
    if storage.is_none() {
        return not_found();
    }
    let storage = storage.unwrap();
    let repository = get_repo_by_name_and_storage(path.0.1.clone(), storage.id, &connection)?;
    if repository.is_none() {
        return not_found();
    }
    let mut repository = repository.unwrap();
    //TODO BAD CODE
    let visibility = Visibility::from_str(path.0.2.as_str()).unwrap();
    repository.security.set_visibility(visibility);
    update_repo(&repository, &connection)?;
    return APIResponse::new(true, Some(repository)).respond(&r);
}

#[post("/api/admin/repository/{storage}/{repo}/modify/security/{what}/{action}/{user}")]
pub async fn update_deployers_readers(
    pool: web::Data<DbPool>,
    r: HttpRequest,
    path: web::Path<(String, String, String, String, String)>,
) -> SiteResponse {
    let connection = pool.get()?;

    let user =
        get_user_by_header(r.headers(), &connection)?;
    if user.is_none() || !user.unwrap().permissions.admin {
        return unauthorized();
    }
    let string = path.0.0.clone();
    let storage = get_storage_by_name(string, &connection)?;
    if storage.is_none() {
        return not_found();
    }
    let storage = storage.unwrap();
    let repository = get_repo_by_name_and_storage(path.0.1.clone(), storage.id, &connection)?;
    if repository.is_none() {
        return not_found();
    }
    let mut repository = repository.unwrap();
    let user = get_user_by_username(path.0.4, &connection)?;
    if user.is_none() {
        return not_found();
    }
    let user = user.unwrap();
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
                _ => return bad_request("Must be Add or Remove")
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
                _ => return bad_request("Must be Add or Remove")
            }
        }
        _ => return  bad_request("Must be Deployers or Readers")
    }
    update_repo(&repository, &connection)?;
    return APIResponse::new(true, Some(repository)).respond(&r);
}

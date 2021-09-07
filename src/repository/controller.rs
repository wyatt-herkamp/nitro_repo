use crate::api_response::APIResponse;


use crate::error::request_error::RequestError;
use crate::error::request_error::RequestError::NotFound;
use crate::repository::action::{
    get_repo_by_name_and_storage, get_repositories_by_storage,
};
use crate::repository::maven::MavenHandler;
use crate::repository::models::Repository;
use crate::repository::repository::{RepoResponse, RepositoryRequest, RepositoryType};

use crate::storage::action::{get_storage_by_name, get_storages};

use crate::system::models::User;

use crate::utils::installed;
use crate::{DbPool};
use actix_files::NamedFile;

use actix_web::web::Bytes;
use actix_web::{delete, get, head, patch, post, put, web, HttpRequest, HttpResponse, Responder};

use serde::{Deserialize, Serialize};
use std::fs::read_to_string;
use std::path::Path;
use crate::repository::repository::RepoResponse::BadRequest;

//

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListRepositories {
    pub repositories: Vec<Repository>,
}

#[get("/storages.json")]
pub async fn browse(
    pool: web::Data<DbPool>,
    _r: HttpRequest,
) -> Result<APIResponse<Vec<String>>, RequestError> {
    let connection = pool.get()?;
    installed(&connection)?;

    let vec = get_storages(&connection)?;
    let mut storages = Vec::new();
    for x in vec {
        storages.push(x.name);
    }
    return Ok(APIResponse::new(true, Some(storages)));
}

#[get("/storages/{storage}.json")]
pub async fn browse_storage(
    pool: web::Data<DbPool>,
    _r: HttpRequest,
    path: web::Path<String>,
) -> Result<APIResponse<Vec<String>>, RequestError> {
    let connection = pool.get()?;
    installed(&connection)?;
    let storage = get_storage_by_name(path.0, &connection)?.ok_or(NotFound)?;
    let vec = get_repositories_by_storage(storage.id, &connection)?;
    let mut repos = Vec::new();
    for x in vec {
        repos.push(x.name);
    }
    return Ok(APIResponse::new(true, Some(repos)));
}

#[get("/storages/{storage}/{repository}/{file:.*}")]
pub async fn get_repository(
    pool: web::Data<DbPool>,
    r: HttpRequest,
    path: web::Path<(String, String, String)>,
) -> Result<HttpResponse, RequestError> {
    let connection = pool.get()?;
    installed(&connection)?;
    let option1 = get_storage_by_name(path.0.0, &connection)?.ok_or(RequestError::NotFound)?;
    let option = get_repo_by_name_and_storage(path.0.1.clone(), option1.id.clone(), &connection)?
        .ok_or(RequestError::NotFound)?;
    let t = option.repo_type.clone();
    let mut string = path.0.2.clone();
    if string.ends_with("api_browse.json") {
        string = string.replace("api_browse.json", "");
    }
    let request = RepositoryRequest {
        //TODO DONT DO THIS
        request: r.clone(),
        storage: option1,
        repository: option,
        value: string,
    };
    let x = match t.as_str() {
        "maven" => MavenHandler::handle_get(request, &connection),
        _ => {
            panic!("Unknown REPO")
        }
    }?;
    return handle_result(x, path.0.2.clone(), r);
}

pub fn handle_result(
    response: RepoResponse,
    url: String,
    r: HttpRequest,
) -> Result<HttpResponse, RequestError> {
    return match response {
        RepoResponse::FileList(files) => {
            if url.ends_with("api_browse.json") {
                Ok(APIResponse::new(true, Some(files)).respond(&r))
            } else {
                let result1 = read_to_string(
                    Path::new(&std::env::var("SITE_DIR").unwrap()).join("browse/[...browse].html"),
                );
                Ok(HttpResponse::Ok()
                    .content_type("text/html")
                    .body(result1.unwrap()))
            }
        }
        RepoResponse::FileResponse(file) => Ok(NamedFile::open(file)?.into_response(&r)?),
        RepoResponse::Ok => Ok(APIResponse::new(true, Some(false)).respond(&r)),
        RepoResponse::NotFound => {
            if url.ends_with(".json") {
                return Err(NotFound);
            } else {
                let result1 =
                    read_to_string(Path::new(&std::env::var("SITE_DIR").unwrap()).join("404.html"));
                Ok(HttpResponse::NotFound()
                    .content_type("text/html")
                    .body(result1.unwrap()))
            }
        }
        RepoResponse::NotAuthorized => {
            return Err(RequestError::NotAuthorized);
        }
        RepoResponse::BadRequest(e) => {
            return Err(RequestError::BadRequest(e.into()));
        }
    };
}

#[post("/storages/{storage}/{repository}/{file:.*}")]
pub async fn post_repository(
    pool: web::Data<DbPool>,
    r: HttpRequest,
    path: web::Path<(String, String, String)>,
    bytes: Bytes,
) -> Result<HttpResponse, RequestError> {
    let connection = pool.get()?;
    installed(&connection)?;
    let option1 = get_storage_by_name(path.0.0, &connection)?.ok_or(RequestError::NotFound)?;
    let option = get_repo_by_name_and_storage(path.0.1.clone(), option1.id.clone(), &connection)?
        .ok_or(RequestError::NotFound)?;
    if !option.settings.active {
        return handle_result(BadRequest("Repo is not active".to_string()), path.0.2.clone(), r);
    }
    let t = option.repo_type.clone();
    let mut string = path.0.2.clone();
    if string.ends_with("api_browse.json") {
        string = string.replace("api_browse.json", "");
    }

    let request = RepositoryRequest {
        //TODO DONT DO THIS
        request: r.clone(),
        storage: option1,
        repository: option,
        value: string,
    };
    let x = match t.as_str() {
        "maven" => MavenHandler::handle_post(request, &connection, bytes),
        _ => {
            panic!("Unknown REPO")
        }
    }?;
    return handle_result(x, path.0.2.clone(), r);
}

#[patch("/storages/{storage}/{repository}/{file:.*}")]
pub async fn patch_repository(
    pool: web::Data<DbPool>,
    r: HttpRequest,
    path: web::Path<(String, String, String)>,
    bytes: Bytes,
) -> Result<HttpResponse, RequestError> {
    let connection = pool.get()?;
    installed(&connection)?;
    let option1 = get_storage_by_name(path.0.0, &connection)?.ok_or(RequestError::NotFound)?;
    let option = get_repo_by_name_and_storage(path.0.1.clone(), option1.id.clone(), &connection)?
        .ok_or(RequestError::NotFound)?;
    if !option.settings.active {
        return handle_result(BadRequest("Repo is not active".to_string()), path.0.2.clone(), r);
    }
    let t = option.repo_type.clone();
    let mut string = path.0.2.clone();
    if string.ends_with("api_browse.json") {
        string = string.replace("api_browse.json", "");
    }

    let request = RepositoryRequest {
        //TODO DONT DO THIS
        request: r.clone(),
        storage: option1,
        repository: option,
        value: string,
    };
    let x = match t.as_str() {
        "maven" => MavenHandler::handle_patch(request, &connection, bytes),
        _ => {
            panic!("Unknown REPO")
        }
    }?;
    return handle_result(x, path.0.2.clone(), r);
}

#[put("/storages/{storage}/{repository}/{file:.*}")]
pub async fn put_repository(
    pool: web::Data<DbPool>,
    r: HttpRequest,
    path: web::Path<(String, String, String)>,
    bytes: Bytes,
) -> Result<HttpResponse, RequestError> {
    let connection = pool.get()?;
    installed(&connection)?;
    let option1 = get_storage_by_name(path.0.0, &connection)?.ok_or(RequestError::NotFound)?;
    let option = get_repo_by_name_and_storage(path.0.1.clone(), option1.id.clone(), &connection)?
        .ok_or(RequestError::NotFound)?;
    if !option.settings.active {
        return handle_result(BadRequest("Repo is not active".to_string()), path.0.2.clone(), r);
    }
    let t = option.repo_type.clone();
    let mut string = path.0.2.clone();
    if string.ends_with("api_browse.json") {
        string = string.replace("api_browse.json", "");
    }

    let request = RepositoryRequest {
        //TODO DONT DO THIS
        request: r.clone(),
        storage: option1,
        repository: option,
        value: string,
    };
    let x = match t.as_str() {
        "maven" => MavenHandler::handle_put(request, &connection, bytes),
        _ => {
            panic!("Unknown REPO")
        }
    }?;
    return handle_result(x, path.0.2.clone(), r);
}

#[head("/storages/{storage}/{repository}/{file:.*}")]
pub async fn head_repository(
    pool: web::Data<DbPool>,
    r: HttpRequest,
    path: web::Path<(String, String, String)>,
) -> Result<HttpResponse, RequestError> {
    let connection = pool.get()?;
    installed(&connection)?;

    let option1 = get_storage_by_name(path.0.0, &connection)?.ok_or(RequestError::NotFound)?;
    let option = get_repo_by_name_and_storage(path.0.1.clone(), option1.id.clone(), &connection)?
        .ok_or(RequestError::NotFound)?;
    if !option.settings.active {
        return handle_result(BadRequest("Repo is not active".to_string()), path.0.2.clone(), r);
    }
    let t = option.repo_type.clone();
    let mut string = path.0.2.clone();
    if string.ends_with("api_browse.json") {
        string = string.replace("api_browse.json", "");
    }

    let request = RepositoryRequest {
        //TODO DONT DO THIS
        request: r.clone(),
        storage: option1,
        repository: option,
        value: string,
    };
    let x = match t.as_str() {
        "maven" => MavenHandler::handle_head(request, &connection),
        _ => {
            panic!("Unknown REPO")
        }
    }?;
    return handle_result(x, path.0.2.clone(), r);
}

use crate::api_response::{APIResponse, SiteResponse};


use crate::repository::action::{get_repo_by_name_and_storage, get_repositories_by_storage};
use crate::repository::maven::MavenHandler;
use crate::repository::models::Repository;
use crate::repository::repository::{RepoResponse, RepositoryRequest, RepositoryType};

use crate::storage::action::{get_storage_by_name, get_storages};

use crate::utils::{get_accept};
use crate::DbPool;
use actix_files::NamedFile;

use actix_web::web::Bytes;
use actix_web::{get, head, patch, post, put, web, HttpRequest, HttpResponse};

use crate::repository::repository::RepoResponse::BadRequest;
use actix_web::http::{ StatusCode};
use serde::{Deserialize, Serialize};
use std::fs::read_to_string;
use std::path::Path;
use crate::error::response::{bad_request, i_am_a_teapot, not_found};

//

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListRepositories {
    pub repositories: Vec<Repository>,
}

#[get("/storages.json")]
pub async fn browse(
    pool: web::Data<DbPool>,
    r: HttpRequest,
) -> SiteResponse {
    let connection = pool.get()?;


    let vec = get_storages(&connection)?;
    let mut storages = Vec::new();
    for x in vec {
        storages.push(x.name);
    }
    return APIResponse::respond_new(Some(storages), &r);
}

#[get("/storages/{storage}.json")]
pub async fn browse_storage(
    pool: web::Data<DbPool>,
    r: HttpRequest,
    path: web::Path<String>,
) -> SiteResponse {
    let connection = pool.get()?;

    let storage = get_storage_by_name(path.0, &connection)?;
    if storage.is_none(){
        return not_found();
    }
    let storage = storage.unwrap();
    let vec = get_repositories_by_storage(storage.id, &connection)?;
    let mut repos = Vec::new();
    for x in vec {
        repos.push(x.name);
    }
    return APIResponse::respond_new(Some(repos), &r);
}

#[get("/storages/{storage}/{repository}/{file:.*}")]
pub async fn get_repository(
    pool: web::Data<DbPool>,
    r: HttpRequest,
    path: web::Path<(String, String, String)>,
) ->SiteResponse{
    let connection = pool.get()?;

       let storage = get_storage_by_name(path.0.0, &connection)?;
    if storage.is_none() {
        return not_found();
    }
    let storage = storage.unwrap();
    let repository = get_repo_by_name_and_storage(path.0.1.clone(), storage.id.clone(), &connection)?;
    if repository.is_none(){
        return not_found();
    }
    let repository = repository.unwrap();

    let t = repository.repo_type.clone();
    let string = path.0.2.clone();

    let request = RepositoryRequest {
        //TODO DONT DO THIS
        request: r.clone(),
        storage: storage,
        repository: repository,
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
/// TODO look into this method
pub fn handle_result(
    response: RepoResponse,
    _url: String,
    r: HttpRequest,
) -> SiteResponse {
    let x = get_accept(r.headers())?.unwrap_or("text/html".to_string());
    return match response {
        RepoResponse::FileList(files) => {
            if x.contains(&"application/json".to_string()) {
               APIResponse::new(true, Some(files)).respond(&r)
            } else {
                let result1 = read_to_string(
                    Path::new(&std::env::var("SITE_DIR").unwrap()).join("browse/[...browse].html"),
                );
                Ok(HttpResponse::Ok()
                    .content_type("text/html")
                    .body(result1.unwrap()))
            }
        }
        RepoResponse::FileResponse(file) => {
            Ok(NamedFile::open(file)?.into_response(&r)?)
        }
        RepoResponse::Ok => APIResponse::new(true, Some(false)).respond(&r),
        RepoResponse::NotFound => {
            if x.contains(&"application/json".to_string()) {
                return not_found();
            } else {
                Ok(HttpResponse::NotFound()
                    .content_type("text/html").body("NOT FOUND"))
            }
        }

        RepoResponse::NotAuthorized => {
            let r = APIResponse::<bool> {
                success: false,
                data: None,
                status_code: Some(401),
            };
            let result = HttpResponse::Ok()
                .status(StatusCode::UNAUTHORIZED)
                .content_type("application/json")
                .header("WWW-Authenticate", "Basic realm=nitro_repo")
                .body(serde_json::to_string(&r).unwrap());
            return Ok(result);
        }
        RepoResponse::BadRequest(e) => {
            return bad_request(e);
        }
        RepoResponse::IAmATeapot(e) => {
            return i_am_a_teapot(e);
        }
        RepoResponse::VersionResponse(value) => APIResponse::new(true, Some(value)).respond(&r),
    };
}

#[post("/storages/{storage}/{repository}/{file:.*}")]
pub async fn post_repository(
    pool: web::Data<DbPool>,
    r: HttpRequest,
    path: web::Path<(String, String, String)>,
    bytes: Bytes,
) -> SiteResponse {
    let connection = pool.get()?;

       let storage = get_storage_by_name(path.0.0, &connection)?;
    if storage.is_none() {
        return not_found();
    }
    let storage = storage.unwrap();
    let repository = get_repo_by_name_and_storage(path.0.1.clone(), storage.id.clone(), &connection)?;
    if repository.is_none(){
        return not_found();
    }
    let repository = repository.unwrap();
    if !repository.settings.active {
        return handle_result(
            BadRequest("Repo is not active".to_string()),
            path.0.2.clone(),
            r,
        );
    }
    let t = repository.repo_type.clone();
    let string = path.0.2.clone();

    let request = RepositoryRequest {
        //TODO DONT DO THIS
        request: r.clone(),
        storage: storage,
        repository: repository,
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
) ->SiteResponse{
    let connection = pool.get()?;

    let storage = get_storage_by_name(path.0.0, &connection)?;
    if storage.is_none() {
        return not_found();
    }
    let storage = storage.unwrap();
    let repository = get_repo_by_name_and_storage(path.0.1.clone(), storage.id.clone(), &connection)?;
    if repository.is_none(){
        return not_found();
    }
    let repository = repository.unwrap();
    if !repository.settings.active {
        return handle_result(
            BadRequest("Repo is not active".to_string()),
            path.0.2.clone(),
            r,
        );
    }
    let t = repository.repo_type.clone();
    let string = path.0.2.clone();

    let request = RepositoryRequest {
        //TODO DONT DO THIS
        request: r.clone(),
        storage: storage,
        repository: repository,
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
) ->SiteResponse{
    let connection = pool.get()?;

       let storage = get_storage_by_name(path.0.0, &connection)?;
    if storage.is_none() {
        return not_found();
    }
    let storage = storage.unwrap();
    let repository = get_repo_by_name_and_storage(path.0.1.clone(), storage.id.clone(), &connection)?;
    if repository.is_none(){
        return not_found();
    }
    let repository = repository.unwrap();
    if !repository.settings.active {
        return handle_result(
            BadRequest("Repo is not active".to_string()),
            path.0.2.clone(),
            r,
        );
    }
    let t = repository.repo_type.clone();
    let string = path.0.2.clone();

    let request = RepositoryRequest {
        //TODO DONT DO THIS
        request: r.clone(),
        storage: storage,
        repository: repository,
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
) ->SiteResponse{
    let connection = pool.get()?;


       let storage = get_storage_by_name(path.0.0, &connection)?;
    if storage.is_none() {
        return not_found();
    }
    let storage = storage.unwrap();
    let repository = get_repo_by_name_and_storage(path.0.1.clone(), storage.id.clone(), &connection)?;
    if repository.is_none(){
        return not_found();
    }
    let repository = repository.unwrap();
    if !repository.settings.active {
        return handle_result(
            BadRequest("Repo is not active".to_string()),
            path.0.2.clone(),
            r,
        );
    }
    let t = repository.repo_type.clone();
    let string = path.0.2.clone();

    let request = RepositoryRequest {
        //TODO DONT DO THIS
        request: r.clone(),
        storage: storage,
        repository: repository,
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

use std::fs::read_to_string;
use std::path::Path;

use actix_files::NamedFile;
use actix_web::http::StatusCode;
use actix_web::web::Bytes;
use actix_web::{get, head, patch, post, put, web, HttpRequest, HttpResponse};
use diesel::MysqlConnection;
use log::{debug, error, trace};
use serde::{Deserialize, Serialize};

use crate::api_response::{APIResponse, SiteResponse};
use crate::error::internal_error::InternalError;
use crate::error::response::{bad_request, i_am_a_teapot, not_found};
use crate::repository::action::{get_repo_by_name_and_storage, get_repositories_by_storage};
use crate::repository::maven::MavenHandler;
use crate::repository::models::Repository;
use crate::repository::npm::NPMHandler;
use crate::repository::repository::{RepoResponse, RepositoryRequest, RepositoryType};
use crate::storage::action::{get_storage_by_name, get_storages};
use crate::utils::get_accept;
use crate::DbPool;

//

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListRepositories {
    pub repositories: Vec<Repository>,
}

fn to_request(
    storage_name: String,
    repo_name: String,
    file: String,
    connection: &MysqlConnection,
) -> Result<RepositoryRequest, InternalError> {
    let storage = get_storage_by_name(&storage_name, &connection)?;
    if storage.is_none() {
        trace!("Storage {} not found", &storage_name);
        return Err(InternalError::NotFound);
    }
    let storage = storage.unwrap();
    let repository = get_repo_by_name_and_storage(&repo_name, &storage.id, &connection)?;
    if repository.is_none() {
        trace!("Repository {} not found", repo_name);
        return Err(InternalError::NotFound);
    }
    let repository = repository.unwrap();

    let request = RepositoryRequest::new(storage, repository, file);
    return Ok(request);
}

#[get("/storages.json")]
pub async fn browse(pool: web::Data<DbPool>, r: HttpRequest) -> SiteResponse {
    let connection = pool.get()?;

    let vec = get_storages(&connection)?;
    let mut storages = Vec::new();
    for x in vec {
        storages.push(x.name);
    }
    APIResponse::respond_new(Some(storages), &r)
}

#[get("/storages/{storage}.json")]
pub async fn browse_storage(
    pool: web::Data<DbPool>,
    r: HttpRequest,
    path: web::Path<String>,
) -> SiteResponse {
    let connection = pool.get()?;

    let string = path.into_inner();
    let storage = get_storage_by_name(&string, &connection)?;
    if storage.is_none() {
        trace!("Storage {} not found", &string);
        return not_found();
    }
    let storage = storage.unwrap();
    let vec = get_repositories_by_storage(&storage.id, &connection)?;
    let mut repos = Vec::new();
    for x in vec {
        repos.push(x.name);
    }
    APIResponse::respond_new(Some(repos), &r)
}

#[get("/storages/{storage}/{repository}/{file:.*}")]
pub async fn get_repository(
    pool: web::Data<DbPool>,
    r: HttpRequest,
    path: web::Path<(String, String, String)>,
) -> SiteResponse {
    let connection = pool.get()?;
    let (storage, repository, file) = path.into_inner();
    let result = to_request(storage, repository, file, &connection);
    if let Err(error) = result {
        return match error {
            InternalError::NotFound => not_found(),
            _ => Err(error),
        };
    }
    let request = result.unwrap();
    let x = match request.repository.repo_type.as_str() {
        "maven" => MavenHandler::handle_get(&request, &r, &connection),
        "npm" => NPMHandler::handle_get(&request, &r, &connection),
        _ => {
            error!("Invalid Repo Type {}", request.repository.repo_type);
            panic!("Unknown REPO")
        }
    }?;
    handle_result(x, request.value, r)
}

/// TODO look into this method
pub fn handle_result(response: RepoResponse, _url: String, r: HttpRequest) -> SiteResponse {
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
        RepoResponse::FileResponse(file) => Ok(NamedFile::open(file)?.into_response(&r)),
        RepoResponse::Ok => APIResponse::new(true, Some(false)).respond(&r),
        RepoResponse::NotFound => {
            if x.contains(&"application/json".to_string()) {
                return not_found();
            } else {
                Ok(HttpResponse::NotFound()
                    .content_type("text/html")
                    .body("NOT FOUND"))
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
                .append_header(("WWW-Authenticate", "Basic realm=nitro_repo"))
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
        RepoResponse::ProjectResponse(project) => APIResponse::new(true, Some(project)).respond(&r),
        RepoResponse::VersionListingResponse(versions) => {
            APIResponse::new(true, Some(versions)).respond(&r)
        }
        RepoResponse::CreatedWithJSON(json) => {
            let result = HttpResponse::Ok()
                .status(StatusCode::CREATED)
                .content_type("application/json")
                .body(json);
            return Ok(result);
        }
        RepoResponse::OkWithJSON(json) => {
            let result = HttpResponse::Ok()
                .status(StatusCode::OK)
                .content_type("application/json")
                .body(json);
            return Ok(result);
        }
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
    let (storage, repository, file) = path.into_inner();
    let result = to_request(storage, repository, file, &connection);
    if let Err(error) = result {
        return match error {
            InternalError::NotFound => not_found(),
            _ => Err(error),
        };
    }
    let request = result.unwrap();
    debug!(
        "POST {} in {}/{}: Route: {}",
        &request.repository.repo_type,
        &request.storage.name,
        &request.repository.name,
        &request.value
    );
    let x = match request.repository.repo_type.as_str() {
        "maven" => MavenHandler::handle_post(&request, &r, &connection, bytes),
        "npm" => NPMHandler::handle_post(&request, &r, &connection, bytes),
        _ => {
            error!("Invalid Repo Type {}", request.repository.repo_type);
            panic!("Unknown REPO")
        }
    }?;
    handle_result(x, request.value, r)
}

#[patch("/storages/{storage}/{repository}/{file:.*}")]
pub async fn patch_repository(
    pool: web::Data<DbPool>,
    r: HttpRequest,
    path: web::Path<(String, String, String)>,
    bytes: Bytes,
) -> SiteResponse {
    let connection = pool.get()?;

    let (storage, repository, file) = path.into_inner();
    let result = to_request(storage, repository, file, &connection);
    if let Err(error) = result {
        return match error {
            InternalError::NotFound => not_found(),
            _ => Err(error),
        };
    }
    let request = result.unwrap();
    debug!(
        "PATCH {} in {}/{}: Route: {}",
        &request.repository.repo_type,
        &request.storage.name,
        &request.repository.name,
        &request.value
    );
    let x = match request.repository.repo_type.as_str() {
        "maven" => MavenHandler::handle_patch(&request, &r, &connection, bytes),
        "npm" => NPMHandler::handle_patch(&request, &r, &connection, bytes),
        _ => {
            panic!("Unknown REPO")
        }
    }?;
    handle_result(x, request.value, r)
}

#[put("/storages/{storage}/{repository}/{file:.*}")]
pub async fn put_repository(
    pool: web::Data<DbPool>,
    r: HttpRequest,
    path: web::Path<(String, String, String)>,
    bytes: Bytes,
) -> SiteResponse {
    let connection = pool.get()?;
    let (storage, repository, file) = path.into_inner();
    let result = to_request(storage, repository, file, &connection);
    if let Err(error) = result {
        return match error {
            InternalError::NotFound => not_found(),
            _ => Err(error),
        };
    }
    let request = result.unwrap();
    debug!(
        "PUT {} in {}/{}: Route: {}",
        &request.repository.repo_type,
        &request.storage.name,
        &request.repository.name,
        &request.value
    );
    let x = match request.repository.repo_type.as_str() {
        "maven" => MavenHandler::handle_put(&request, &r, &connection, bytes),
        "npm" => NPMHandler::handle_put(&request, &r, &connection, bytes),
        _ => {
            error!("Invalid Repo Type {}", request.repository.repo_type);
            panic!("Unknown REPO")
        }
    }?;
    handle_result(x, request.value, r)
}

#[head("/storages/{storage}/{repository}/{file:.*}")]
pub async fn head_repository(
    pool: web::Data<DbPool>,
    r: HttpRequest,
    path: web::Path<(String, String, String)>,
) -> SiteResponse {
    let connection = pool.get()?;

    let (storage, repository, file) = path.into_inner();
    let result = to_request(storage, repository, file, &connection);
    if let Err(error) = result {
        return match error {
            InternalError::NotFound => not_found(),
            _ => Err(error),
        };
    }
    let request = result.unwrap();
    debug!(
        "HEAD {} in {}/{}: Route: {}",
        &request.repository.repo_type,
        &request.storage.name,
        &request.repository.name,
        &request.value
    );
    let x = match request.repository.repo_type.as_str() {
        "maven" => MavenHandler::handle_head(&request, &r, &connection),
        "npm" => NPMHandler::handle_head(&request, &r, &connection),
        _ => {
            error!("Invalid Repo Type {}", request.repository.repo_type);
            panic!("Unknown REPO")
        }
    }?;
    handle_result(x, request.value, r)
}

use std::fs::read_to_string;
use std::path::Path;

use actix_files::NamedFile;
use actix_web::http::StatusCode;
use actix_web::web::Bytes;
use actix_web::{get, head, patch, post, put, web, HttpRequest, HttpResponse};
use log::{debug, error, trace};
use serde::{Deserialize, Serialize};

use crate::api_response::{APIResponse, SiteResponse};
use crate::database::DbPool;
use crate::error::internal_error::InternalError;
use crate::error::response::{bad_request, i_am_a_teapot, not_found};
use crate::NitroRepoData;
use crate::repository::maven::MavenHandler;
use crate::repository::models::Repository;
use crate::repository::npm::NPMHandler;
use crate::repository::types::{RepoResponse, RepositoryRequest, RepositoryType};
use crate::utils::get_accept;

//

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListRepositories {
    pub repositories: Vec<Repository>,
}

pub fn to_request(
    storage_name: String,
    repo_name: String,
    file: String,
    site: NitroRepoData,
) -> Result<RepositoryRequest, InternalError> {
    let storages = site.storages.lock().unwrap();
    let storage = storages.get(&storage_name);
    if storage.is_none() {
        trace!("Storage {} not found", &storage_name);
        return Err(InternalError::NotFound);
    }
    let storage = storage.unwrap().clone();
    let repository = storage.get_repository(&repo_name)?;
    if repository.is_none() {
        trace!("Repository {} not found", repo_name);
        return Err(InternalError::NotFound);
    }
    let repository = repository.unwrap();

    let request = RepositoryRequest {
        storage,
        repository,
        value: file,
    };
    Ok(request)
}

#[get("/storages.json")]
pub async fn browse(site: NitroRepoData, r: HttpRequest) -> SiteResponse {
    let mut storages = Vec::new();
    for (name, _) in site.storages.lock().unwrap().iter() {
        storages.push(name.clone());
    }
    APIResponse::respond_new(Some(storages), &r)
}

#[get("/storages/{storage}.json")]
pub async fn browse_storage(
    site: NitroRepoData,
    r: HttpRequest,
    path: web::Path<String>,
) -> SiteResponse {
    let string = path.into_inner();
    let storages = site.storages.lock().unwrap();

    let storage = storages.get(&string);
    if storage.is_none() {
        trace!("Storage {} not found", &string);
        return Err(InternalError::NotFound);
    }
    let storage = storage.unwrap();
    let  repos = storage.get_repositories()?;
    APIResponse::respond_new(Some(repos), &r)
}


#[get("/storages/{storage}/{repository}/{file:.*}")]
pub async fn get_repository(
    pool: web::Data<DbPool>,
    site: NitroRepoData,
    r: HttpRequest,
    path: web::Path<(String, String, String)>,
) -> SiteResponse {
    let connection = pool.get()?;
    let (storage, repository, file) = path.into_inner();
    let result = to_request(storage, repository, file, site);
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
    let x = get_accept(r.headers())?.unwrap_or_else(|| "text/html".to_string());
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
        RepoResponse::NitroVersionListingResponse(value) => {
            APIResponse::new(true, Some(value)).respond(&r)
        }
        RepoResponse::NitroVersionResponse(value) => {
            APIResponse::new(true, Some(value)).respond(&r)
        }
        RepoResponse::NitroProjectResponse(value) => {
            APIResponse::new(true, Some(value)).respond(&r)
        }
    };
}

#[post("/storages/{storage}/{repository}/{file:.*}")]
pub async fn post_repository(
    pool: web::Data<DbPool>,
    site: NitroRepoData,
    r: HttpRequest,
    path: web::Path<(String, String, String)>,
    bytes: Bytes,
) -> SiteResponse {
    let connection = pool.get()?;
    let (storage, repository, file) = path.into_inner();
    let result = to_request(storage, repository, file, site);
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
    site: NitroRepoData,
    r: HttpRequest,
    path: web::Path<(String, String, String)>,
    bytes: Bytes,
) -> SiteResponse {
    let connection = pool.get()?;

    let (storage, repository, file) = path.into_inner();
    let result = to_request(storage, repository, file, site);
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
    pool: web::Data<DbPool>, site: NitroRepoData,
    r: HttpRequest,
    path: web::Path<(String, String, String)>,
    bytes: Bytes,
) -> SiteResponse {
    let connection = pool.get()?;
    let (storage, repository, file) = path.into_inner();
    let result = to_request(storage, repository, file, site);
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
    pool: web::Data<DbPool>, site: NitroRepoData,
    r: HttpRequest,
    path: web::Path<(String, String, String)>,
) -> SiteResponse {
    let connection = pool.get()?;

    let (storage, repository, file) = path.into_inner();
    let result = to_request(storage, repository, file, site);
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

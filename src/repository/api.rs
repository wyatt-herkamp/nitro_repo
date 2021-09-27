use crate::api_response::APIResponse;


use crate::error::request_error::RequestError;
use crate::error::request_error::RequestError::{NotFound, NotAuthorized};
use crate::repository::action::{
    get_repo_by_name_and_storage, get_repositories_by_storage,
};
use crate::repository::maven::MavenHandler;
use crate::repository::models::{Repository, Visibility};
use crate::repository::repository::{RepoResponse, RepositoryRequest, RepositoryType};

use crate::storage::action::{get_storage_by_name, get_storages};

use crate::system::models::User;

use crate::utils::{installed, get_accept};
use crate::{DbPool};
use actix_files::NamedFile;

use actix_web::web::Bytes;
use actix_web::{delete, get, head, patch, post, put, web, HttpRequest, HttpResponse, Responder, HttpMessage};

use serde::{Deserialize, Serialize};
use std::fs::read_to_string;
use std::path::Path;
use crate::repository::repository::RepoResponse::BadRequest;
use actix_web::http::StatusCode;
use crate::system::utils::can_read_basic_auth;
use crate::repository::controller::handle_result;

//

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListRepositories {
    pub repositories: Vec<Repository>,
}

#[get("/api/version/{storage}/{repository}/{file:.*}")]
pub async fn get_versions(
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

    let request = RepositoryRequest {
        //TODO DONT DO THIS
        request: r.clone(),
        storage: option1,
        repository: option,
        value: string,
    };
    let x = match t.as_str() {
        "maven" => MavenHandler::handle_versions(request, &connection),
        _ => {
            panic!("Unknown REPO")
        }
    }?;
    return handle_result(x, path.0.2.clone(), r);
}
#[get("/api/about/{storage}/{repository}/{file:.*}")]
pub async fn get_about(
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
    return Err(NotFound);
}

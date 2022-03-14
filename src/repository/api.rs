use actix_web::{get, web, HttpRequest};
use serde::{Deserialize, Serialize};

use crate::api_response::SiteResponse;
use crate::database::DbPool;

use crate::repository::controller::{handle_result, to_request};
use crate::repository::maven::MavenHandler;
use crate::repository::models::Repository;
use crate::repository::npm::NPMHandler;
use crate::repository::types::RepositoryType;

//

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListRepositories {
    pub repositories: Vec<Repository>,
}

#[get("/api/versions/{storage}/{repository}/{file:.*}")]
pub async fn get_versions(
    pool: web::Data<DbPool>,
    r: HttpRequest,
    path: web::Path<(String, String, String)>,
) -> SiteResponse {
    let (storage, repository, file) = path.into_inner();
    let connection = pool.get()?;

    let request = to_request(storage, repository, file, &connection)?;

    let x = match request.repository.repo_type.as_str() {
        "maven" => MavenHandler::handle_versions(&request, &r, &connection),
        "npm" => NPMHandler::handle_versions(&request, &r, &connection),
        _ => {
            panic!("Unknown REPO")
        }
    }?;
    handle_result(x, request.value, r)
}

#[get("/api/project/{storage}/{repository}/{file:.*}")]
pub async fn get_project(
    pool: web::Data<DbPool>,
    r: HttpRequest,
    path: web::Path<(String, String, String)>,
) -> SiteResponse {
    let (storage, repository, file) = path.into_inner();
    println!("{}",file);
    let connection = pool.get()?;

    let request = to_request(storage, repository, file, &connection)?;

    let x = match request.repository.repo_type.as_str() {
        "maven" => MavenHandler::handle_project(&request, &r, &connection),
        _ => {
            panic!("Unknown REPO")
        }
    }?;
    handle_result(x, request.value, r)
}

#[get("/api/version/{storage}/{repository}/{project}/{version}")]
pub async fn get_version(
    pool: web::Data<DbPool>,
    r: HttpRequest,
    path: web::Path<(String, String, String, String)>,
) -> SiteResponse {
    let (storage, repository, project, version) = path.into_inner();
    let connection = pool.get()?;

    let request = to_request(storage, repository, project, &connection)?;

    let x = match request.repository.repo_type.as_str() {
        "maven" => MavenHandler::handle_version(&request, version, &r, &connection),
        _ => {
            panic!("Unknown REPO")
        }
    }?;
    handle_result(x, request.value, r)
}

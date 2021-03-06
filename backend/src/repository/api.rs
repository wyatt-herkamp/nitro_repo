use actix_web::{get, web, HttpRequest};
use serde::{Deserialize, Serialize};

use crate::api_response::SiteResponse;
use crate::database::DbPool;
use crate::NitroRepoData;

use crate::repository::controller::{handle_result, to_request};
use crate::repository::maven::MavenHandler;
use crate::repository::models::Repository;
use crate::repository::npm::NPMHandler;
use crate::repository::types::RepositoryHandler;
use crate::repository::types::RepositoryType::{Maven, NPM};

//

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListRepositories {
    pub repositories: Vec<Repository>,
}

#[get("/api/versions/{storage}/{repository}/{file:.*}")]
pub async fn get_versions(
    pool: web::Data<DbPool>,
    site: NitroRepoData,
    r: HttpRequest,
    path: web::Path<(String, String, String)>,
) -> SiteResponse {
    let (storage, repository, file) = path.into_inner();
    let connection = pool.get()?;

    let request = to_request(storage, repository, file, site).await?;

    let x = match request.repository.repo_type {
        Maven(_) => { MavenHandler::handle_versions(&request, &r, &connection) }
        NPM(_) => { NPMHandler::handle_versions(&request, &r, &connection) }
    }?;
    handle_result(x, request.value, r)
}

#[get("/api/project/{storage}/{repository}/{file:.*}")]
pub async fn get_project(
    pool: web::Data<DbPool>,
    site: NitroRepoData,
    r: HttpRequest,
    path: web::Path<(String, String, String)>,
) -> SiteResponse {
    let (storage, repository, file) = path.into_inner();
    let connection = pool.get()?;

    let request = to_request(storage, repository, file, site).await?;

    let x = match request.repository.repo_type {
        Maven(_) => { MavenHandler::handle_project(&request, &r, &connection) }
        NPM(_) => { NPMHandler::handle_project(&request, &r, &connection) }
    }?;
    handle_result(x, request.value, r)
}

#[get("/api/version/{storage}/{repository}/{project}/{version}")]
pub async fn get_version(
    pool: web::Data<DbPool>,
    site: NitroRepoData,
    r: HttpRequest,
    path: web::Path<(String, String, String, String)>,
) -> SiteResponse {
    let (storage, repository, project, version) = path.into_inner();
    let connection = pool.get()?;

    let request = to_request(storage, repository, project, site).await?;


    let x = match request.repository.repo_type {
        Maven(_) => { MavenHandler::handle_version(&request, version, &r, &connection) }
        NPM(_) => { NPMHandler::handle_version(&request, version, &r, &connection) }
    }?;
    handle_result(x, request.value, r)
}

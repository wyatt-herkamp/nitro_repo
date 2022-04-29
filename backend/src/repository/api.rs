use actix_web::{get, web, HttpRequest};
use sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};

use crate::api_response::SiteResponse;
use crate::NitroRepoData;

use crate::authentication::Authentication;
use crate::repository::controller::{handle_result, to_request};
use crate::repository::models::Repository;
use crate::storage::StorageManager;
use crate::system::permissions::options::CanIDo;
use crate::system::user::UserModel;
//

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListRepositories {
    pub repositories: Vec<Repository>,
}

#[get("/api/versions/{storage}/{repository}/{file:.*}")]
pub async fn get_versions(
    connection: web::Data<DatabaseConnection>,
    site: NitroRepoData,
    r: HttpRequest,
    auth: Authentication,
    path: web::Path<(String, String, String)>,
    storages: web::Data<StorageManager>,
) -> SiteResponse {
    let (storage, repository, file) = path.into_inner();

    let request = to_request(storage, repository, file, site, storages).await?;
    let caller: UserModel = auth.get_user(&connection).await??;
    caller.can_read_from(&request.repository)?;
    let x = request
        .repository
        .repo_type
        .handle_versions(&request, &r, &connection)
        .await?;
    handle_result(x, request.value, r)
}

#[get("/api/project/{storage}/{repository}/{file:.*}")]
pub async fn get_project(
    connection: web::Data<DatabaseConnection>,
    site: NitroRepoData,
    r: HttpRequest,
    auth: Authentication,
    path: web::Path<(String, String, String)>,
    storages: web::Data<StorageManager>,
) -> SiteResponse {
    let (storage, repository, file) = path.into_inner();

    let request = to_request(storage, repository, file, site, storages).await?;
    let caller: UserModel = auth.get_user(&connection).await??;
    caller.can_read_from(&request.repository)?;
    let x = request
        .repository
        .repo_type
        .handle_project(&request, &r, &connection)
        .await?;

    handle_result(x, request.value, r)
}

#[get("/api/version/{storage}/{repository}/{project}/{version}")]
pub async fn get_version(
    connection: web::Data<DatabaseConnection>,
    site: NitroRepoData,
    r: HttpRequest,
    auth: Authentication,
    storages: web::Data<StorageManager>,
    path: web::Path<(String, String, String, String)>,
) -> SiteResponse {
    let (storage, repository, project, version) = path.into_inner();

    let request = to_request(storage, repository, project, site, storages).await?;
    let caller: UserModel = auth.get_user(&connection).await??;
    caller.can_read_from(&request.repository)?;
    let x = request
        .repository
        .repo_type
        .handle_version(&request, version, &r, &connection)
        .await?;
    handle_result(x, request.value, r)
}

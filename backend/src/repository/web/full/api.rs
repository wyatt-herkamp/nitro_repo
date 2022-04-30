use actix_web::{get, web, HttpRequest};
use sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};

use crate::api_response::{APIResponse, SiteResponse};
use crate::NitroRepoData;

use crate::authentication::Authentication;
use crate::error::response::not_found;
use crate::repository::models::Repository;
use crate::repository::public::PublicRepositoryResponse;
use crate::repository::settings::security::Visibility;
use crate::repository::web::full::{handle_result, to_request};
use crate::storage::multi::MultiStorageController;
use crate::system::permissions::options::CanIDo;
use crate::system::user::UserModel;

#[get("/api/versions/{storage}/{repository}/{file:.*}")]
pub async fn get_versions(
    connection: web::Data<DatabaseConnection>,
    site: NitroRepoData,
    r: HttpRequest,
    auth: Authentication,
    path: web::Path<(String, String, String)>,
    storages: web::Data<MultiStorageController>,
) -> SiteResponse {
    let (storage, repository, file) = path.into_inner();

    let request = to_request(storage, repository, file, storages).await?;
    let caller: UserModel = auth.get_user(&connection).await??;
    caller.can_read_from(&request.repository)?;
    let x = request
        .repository
        .repo_type
        .handle_versions(&request, &r, &connection)
        .await?;
    handle_result(x, r)
}

#[get("/api/project/{storage}/{repository}/{file:.*}")]
pub async fn get_project(
    connection: web::Data<DatabaseConnection>,
    site: NitroRepoData,
    r: HttpRequest,
    auth: Authentication,
    path: web::Path<(String, String, String)>,
    storages: web::Data<MultiStorageController>,
) -> SiteResponse {
    let (storage, repository, file) = path.into_inner();

    let request = to_request(storage, repository, file, storages).await?;
    let caller: UserModel = auth.get_user(&connection).await??;
    caller.can_read_from(&request.repository)?;
    let x = request
        .repository
        .repo_type
        .handle_project(&request, &r, &connection)
        .await?;

    handle_result(x,  r)
}

#[get("/api/version/{storage}/{repository}/{project}/{version}")]
pub async fn get_version(
    connection: web::Data<DatabaseConnection>,
    site: NitroRepoData,
    r: HttpRequest,
    auth: Authentication,
    storages: web::Data<MultiStorageController>,
    path: web::Path<(String, String, String, String)>,
) -> SiteResponse {
    let (storage, repository, project, version) = path.into_inner();

    let request = to_request(storage, repository, project, storages).await?;
    let caller: UserModel = auth.get_user(&connection).await??;
    caller.can_read_from(&request.repository)?;
    let x = request
        .repository
        .repo_type
        .handle_version(&request, version, &r, &connection)
        .await?;
    handle_result(x, r)
}
#[get("/api/repositories/get/{storage}/{repo}")]
pub async fn get_repo(
    connection: web::Data<DatabaseConnection>,
    _site: NitroRepoData,
    storages: web::Data<MultiStorageController>,
    r: HttpRequest,
    auth: Authentication,
    path: web::Path<(String, String)>,
) -> SiteResponse {
    let (storage, repo) = path.into_inner();
    if let Some(storage) = storages.get_storage_by_name(&storage).await? {
        let option = storage.get_repository(&repo).await?;
        if let Some(repository) = option {
            if repository.security.visibility.eq(&Visibility::Private) {
                let caller: UserModel = auth.get_user(&connection).await??;
                caller.can_read_from(&repository)?;
            }
            return APIResponse::respond_new(Some(PublicRepositoryResponse::from(repository)), &r);
        }
    }
    not_found()
}

use actix_web::{HttpRequest, HttpResponse, web as ActixWeb, web};
use actix_web::http::StatusCode;
use log::trace;
use crate::api_response::{APIResponse, SiteResponse};
use crate::error::internal_error::InternalError;
use crate::error::response::{bad_request, i_am_a_teapot, not_found};
use crate::NitroRepoData;
use crate::repository::error::RepositoryError;
use crate::repository::handler::Repository;
use crate::repository::response::{RepoResponse};
use crate::storage::models::Storage;
use crate::storage::multi::MultiStorageController;
use crate::utils::get_accept;


pub mod admin;

/**
pub fn init(cfg: &mut ActixWeb::ServiceConfig) {
    cfg.service(controller::browse_storage)
        .service(ActixWeb::resource(["/storages/", "/storages"]).to(controller::browse))
        .service(
            ActixWeb::resource([
                "/storages/{storage}/{repository}",
                "/storages/{storage}/{repository}/{file:.*}",
                "/storages/{storage}/{repository}/",
            ])
                .route(ActixWeb::get().to(controller::get_repository))
                .route(ActixWeb::put().to(controller::put_repository))
                .route(ActixWeb::head().to(controller::head_repository))
                .route(ActixWeb::patch().to(controller::patch_repository))
                .route(ActixWeb::post().to(controller::post_repository)),
        )
        .service(api::get_versions)
        .service(api::get_version)
        .service(api::get_project)
        .service(badge::badge)
        .service(api::get_repo);
}
 **/

pub async fn to_request(
    storage_name: String,
    repo_name: String,
    storages: web::Data<MultiStorageController>,
) -> Result<(Storage, Repository), RepositoryError> {
    let storage = storages.get_storage_by_name(&storage_name).await?;
    if storage.is_none() {
        trace!("Storage {} not found", &storage_name);
        return Err(RepositoryError::RequestError("Storage Not Found".to_string(), StatusCode::NOT_FOUND));
    }
    let storage = storage.unwrap().clone();
    let repository = Repository::load(&storage, &repo_name).await?;
    if repository.is_none() {
        trace!("Repository {} not found", repo_name);
        return Err(RepositoryError::RequestError("Repository Not Found".to_string(), StatusCode::NOT_FOUND));
    }
    let repository = repository.unwrap();


    Ok((storage, repository))
}

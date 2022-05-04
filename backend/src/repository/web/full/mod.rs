use crate::repository::data::RepositoryType;
use actix_web::http::StatusCode;
use actix_web::web;
use log::trace;
use serde::de::Unexpected::Option;
use std::ops::Deref;

use crate::repository::error::RepositoryError;
use crate::repository::handler::RepositoryHandler;
use crate::repository::maven::MavenHandler;
use crate::repository::npm::NPMHandler;

use crate::storage::models::Storage;
use crate::storage::multi::MultiStorageController;

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

pub async fn to_request<'a>(
    storage_name: String,
    repo_name: String,
    storages: &'a web::Data<MultiStorageController>,
) -> Result<Box<dyn RepositoryHandler<'a> + 'a>, RepositoryError> {
    let storage = storages.get_storage_by_name(&storage_name).await?;
    if storage.is_none() {
        trace!("Storage {} not found", &storage_name);
        return Err(RepositoryError::RequestError(
            "Storage Not Found".to_string(),
            StatusCode::NOT_FOUND,
        ));
    }
    let storage = storage.unwrap();
    let repository = storage.get_repository(&repo_name).await?;
    if repository.is_none() {
        trace!("Repository {} not found", repo_name);
        return Err(RepositoryError::RequestError(
            "Repository Not Found".to_string(),
            StatusCode::NOT_FOUND,
        ));
    }
    let repository = repository.unwrap().clone();

    return match repository.repository_type {
        RepositoryType::Maven => {
            let handler = MavenHandler::create(repository, storage)?;
            Ok(Box::new(handler))
        }
        RepositoryType::NPM => {
            let handler = NPMHandler::create(repository, storage)?;
            Ok(Box::new(handler))
        }
    };
}

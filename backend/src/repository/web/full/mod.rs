use crate::api_response::APIResponse;
use crate::repository::data::RepositoryType;
use actix_web::http::StatusCode;
use actix_web::web;
use log::trace;

use crate::repository::handler::RepositoryHandler;
use crate::repository::maven::MavenHandler;
use crate::repository::npm::NPMHandler;

use crate::storage::multi::MultiStorageController;
pub mod admin;
pub mod controller;

pub async fn to_request<'a>(
    storage_name: &str,
    repo_name: &str,
    storages: &'a web::Data<MultiStorageController>,
) -> Result<Box<dyn RepositoryHandler<'a> + 'a>, APIResponse> {
    let storage = storages.get_storage_by_name(storage_name).await?;
    if storage.is_none() {
        trace!("Storage {} not found", &storage_name);
        return Err(APIResponse::not_found());
    }
    let storage = storage.unwrap();
    let repository = storage.get_repository(repo_name).await?;
    if repository.is_none() {
        trace!("Repository {} not found", repo_name);
        return Err(APIResponse::not_found());
    }
    let repository = repository.unwrap().clone();

    return match repository.repository_type {
        RepositoryType::Maven => {
            let handler = MavenHandler::create(repository, storage);
            Ok(Box::new(handler))
        }
        RepositoryType::NPM => {
            let handler = NPMHandler::create(repository, storage);
            Ok(Box::new(handler))
        }
    };
}

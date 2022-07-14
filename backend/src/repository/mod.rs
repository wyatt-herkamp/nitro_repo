use std::ops::Deref;

use tokio::sync::RwLockReadGuard;

use crate::error::internal_error::InternalError;
use crate::repository::ci::CIHandler;
use crate::repository::docker::DockerHandler;
use crate::repository::handler::DynamicRepositoryHandler;
use crate::repository::maven::MavenHandler;
use crate::repository::npm::NPMHandler;
use crate::repository::raw::RawHandler;
use crate::repository::settings::RepositoryType;
use crate::storage::models::Storage;

pub mod ci;
pub mod docker;
pub mod frontend;
pub mod handler;
pub mod maven;
pub mod nitro;
pub mod npm;
pub mod raw;
pub mod response;
pub mod settings;
pub mod web;

pub static REPOSITORY_CONF: &str = "repository.nitro_repo";
pub static REPOSITORY_CONF_FOLDER: &str = ".config.nitro_repo";
pub static REPOSITORY_CONF_BAK: &str = "repository.nitro_repo.bak";

pub async fn get_repository_handler<'a, StorageType: Storage>(
    storage: RwLockReadGuard<'a, StorageType>,
    repository: &'a str,
) -> Result<Option<DynamicRepositoryHandler<'a, StorageType>>, InternalError> {
    let value = storage.get_repository(repository).await?;
    if value.is_none() {
        return Ok(None);
    }
    let repository_config = value.unwrap().deref().clone();
    match repository_config.repository_type {
        RepositoryType::Maven => Ok(Some(DynamicRepositoryHandler::Maven(MavenHandler::create(
            repository_config,
            storage,
        )))),
        RepositoryType::NPM => Ok(Some(DynamicRepositoryHandler::NPM(NPMHandler::create(
            repository_config,
            storage,
        )))),
        RepositoryType::CI => Ok(Some(DynamicRepositoryHandler::CI(CIHandler::create(
            repository_config,
            storage,
        )))),
        RepositoryType::Docker => Ok(Some(DynamicRepositoryHandler::Docker(
            DockerHandler::create(repository_config, storage),
        ))),
        RepositoryType::Raw => Ok(Some(DynamicRepositoryHandler::Raw(RawHandler::create(
            repository_config,
            storage,
        )))),
    }
}

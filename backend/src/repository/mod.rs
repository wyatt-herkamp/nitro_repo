use std::ops::Deref;

use tokio::sync::RwLockReadGuard;

use crate::error::internal_error::InternalError;
use crate::repository::ci::CIHandler;
use crate::repository::docker::DockerHandler;
use crate::repository::handler::DynamicRepositoryHandler;
use crate::repository::maven::MavenHandler;
use crate::repository::nitro::dynamic::DynamicNitroRepositoryHandler;
use crate::repository::npm::NPMHandler;
use crate::repository::raw::RawHandler;
use crate::repository::settings::{RepositoryConfig, RepositoryType};
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
macro_rules! repository_handler {
    ($($repository_type:ident, $repository:tt),*) => {
        #[inline]
        pub async fn get_repository_handler<StorageType: Storage>(
            storage: RwLockReadGuard<'_, StorageType>,
            repository_config: RepositoryConfig,
        ) -> Result<Option<DynamicRepositoryHandler<StorageType>>, InternalError> {
            match repository_config.repository_type {
                $(
                    RepositoryType::$repository_type => {
                        let handler = $repository::create(repository_config, storage);
                        Ok(Some(DynamicRepositoryHandler::$repository_type(handler)))
                    },
                )*
            }
        }
    };
}
repository_handler!(
    NPM,
    NPMHandler,
    Maven,
    MavenHandler,
    Docker,
    DockerHandler,
    CI,
    CIHandler,
    Raw,
    RawHandler
);

pub enum NitroRepoHandler<'a, StorageType: Storage> {
    Supported(DynamicNitroRepositoryHandler<'a, StorageType>),
    /// it is a teapot! [Teapot](https://http.cat/418)
    Unsupported(RepositoryConfig),
}

macro_rules! gen_nitro_repo_handler {
    ($($repository_type:ident, $repository:tt),*) => {
        #[inline]
        pub async fn nitro_repo_handler<StorageType: Storage>(
            storage: RwLockReadGuard<'_, StorageType>,
            repository_config: RepositoryConfig,
        ) -> Result<Option<NitroRepoHandler<StorageType>>, InternalError> {
            match repository_config.repository_type {
                $(
                    RepositoryType::$repository_type => {
                        let handler = $repository::create(repository_config, storage);
                        Ok(Some(NitroRepoHandler::Supported(DynamicNitroRepositoryHandler::$repository_type(handler))))
                    },
                )*
                _ => Ok(Some(NitroRepoHandler::Unsupported(repository_config))),

            }
        }
    };
}
gen_nitro_repo_handler!(Maven, MavenHandler);

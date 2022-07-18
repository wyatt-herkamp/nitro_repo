use crate::error::internal_error::InternalError;
use crate::repository::maven::MavenHandler;
use crate::repository::nitro::nitro_repository::NitroRepositoryHandler;
use crate::repository::nitro::{NitroFileResponse, NitroRepoVersions, VersionData};
use crate::repository::npm::NPMHandler;
use crate::repository::response::Project;
use crate::repository::settings::RepositoryConfig;
use crate::repository::settings::RepositoryType;
use crate::storage::file::StorageDirectoryResponse;
use crate::storage::models::Storage;
use crate::system::user::database::Model;
use async_trait::async_trait;
use tokio::sync::RwLockReadGuard;

/// Impls the NitroRepository for the DynamicNitroRepositoryHandler
/// # Arguments
/// Array<Name> of the Repository Types
macro_rules! nitro_repo_handler {
    ($($name: ident, $ty:tt),*) => {

        pub enum DynamicNitroRepositoryHandler<'a, StorageType: Storage> {
            $(
                $name($ty<'a, StorageType>),
            )*
        }
        #[inline]
        pub async fn nitro_repo_handler<StorageType: Storage>(
            storage: RwLockReadGuard<'_, StorageType>,
            repository_config: RepositoryConfig,
        ) -> Result<Option<NitroRepoHandler<StorageType>>, InternalError> {
            match repository_config.repository_type {
                $(
                    RepositoryType::$name => {
                        let handler = $ty::create(repository_config, storage);
                        Ok(Some(NitroRepoHandler::Supported(DynamicNitroRepositoryHandler::$name(handler))))
                    },
                )*
                _ => Ok(Some(NitroRepoHandler::Unsupported(repository_config))),

            }
        }
        #[async_trait]
        impl<'a, StorageType: Storage> NitroRepositoryHandler<StorageType>
            for DynamicNitroRepositoryHandler<'a, StorageType>
        {
            fn parse_project_to_directory<S: Into<String>>(value: S) -> String {
                panic!("Parse Should be implemented in the Dynamic Nitro Repository Handler");
            }

            fn storage(&self) -> &StorageType {
                match self {
                 $(DynamicNitroRepositoryHandler::$name(handler) => handler.storage(),)*
                }
            }

            fn repository(&self) -> &RepositoryConfig {
                match self {
                    $(DynamicNitroRepositoryHandler::$name(handler) =>handler.repository() ,)*
                }
            }
            async fn get_versions(
                &self,
                project: &str,
            ) -> Result<Option<NitroRepoVersions>, InternalError> {
                match self {
                    $(DynamicNitroRepositoryHandler::$name(handler) => handler.get_versions(project).await,)*
                }
            }

            async fn get_project_specific_version(
                &self,
                project: &str,
                version: &str,
            ) -> Result<Option<Project>, InternalError> {
                match self {
                    $(DynamicNitroRepositoryHandler::$name(handler) => handler.get_project_specific_version(project, version).await,)*
                }
            }
            async fn get_project_latest(
                &self,
                project: &str,
            ) -> Result<Option<Project>, InternalError> {
                match self {
                    $(DynamicNitroRepositoryHandler::$name(handler) => handler.get_project_latest(project).await,)*
                }
            }
            async fn latest_version(&self, project: &str) -> Result<Option<String>, InternalError> {
                match self {
                    $(DynamicNitroRepositoryHandler::$name(handler) => handler.latest_version(project).await,)*
                }
            }
            async fn process_storage_files(
                &self,
                directory: StorageDirectoryResponse,
                requested_dir: &str,
            ) -> Result<NitroFileResponse, InternalError> {
                match self {
                    $(DynamicNitroRepositoryHandler::$name(handler) => handler.process_storage_files(directory, requested_dir).await,)*
                }
            }
            async fn post_deploy(
                &self,
                project_folder: String,
                version_folder: String,
                model: Model,
                version_data: VersionData,
            ) -> Result<(), InternalError> {
                match self {
                    $(DynamicNitroRepositoryHandler::$name(handler) => handler.post_deploy(project_folder, version_folder, model, version_data).await,)*
                }
            }
        }
    };
}
pub enum NitroRepoHandler<'a, StorageType: Storage> {
    Supported(DynamicNitroRepositoryHandler<'a, StorageType>),
    /// it is a teapot! [Teapot](https://http.cat/418)
    Unsupported(RepositoryConfig),
}

nitro_repo_handler!(Maven, MavenHandler, NPM, NPMHandler);

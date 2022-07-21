/// Impls the NitroRepository for the DynamicNitroRepositoryHandler
/// # Arguments
/// Array<Name> of the Repository Types
macro_rules! nitro_repo_handler {
    ($v:ident, $($name: ident, $ty:tt),*) => {
        #[async_trait::async_trait]
        impl<'a, StorageType: Storage> crate::repository::nitro::nitro_repository::NitroRepositoryHandler<StorageType>
            for $v<StorageType>
        {
            fn parse_project_to_directory<S: Into<String>>(_value: S) -> String {
                panic!("Parse Should be implemented in the Dynamic Nitro Repository Handler");
            }
            fn supports_nitro(&self) -> bool {
                match self {
                    $($v::$name(handler) => handler.supports_nitro(),)*
                    _ => false,
                }
            }

            async fn get_versions(
                &self,
                project: &str,
            ) -> Result<Option<crate::repository::nitro::NitroRepoVersions>, InternalError> {
                match self {
                    $($v::$name(handler) => handler.get_versions(project).await,)*
                    _ => unsafe{ std::hint::unreachable_unchecked() },

                }
            }

            async fn get_project_specific_version(
                &self,
                project: &str,
                version: &str,
            ) -> Result<Option<crate::repository::response::Project>, InternalError> {
                match self {
                    $($v::$name(handler) => handler.get_project_specific_version(project, version).await,)*
                    _ => unsafe{ std::hint::unreachable_unchecked() },
                }
            }
            async fn get_project_latest(
                &self,
                project: &str,
            ) -> Result<Option<crate::repository::response::Project>, InternalError> {
                match self {
                    $($v::$name(handler) => handler.get_project_latest(project).await,)*
                    _ => unsafe{ std::hint::unreachable_unchecked() },

                }
            }
            async fn latest_version(&self, project: &str) -> Result<Option<String>, InternalError> {
                match self {
                    $($v::$name(handler) => handler.latest_version(project).await,)*
                    _ => unsafe{ std::hint::unreachable_unchecked() },

                }
            }
            async fn process_storage_files(
                &self,
                directory: crate::storage::file::StorageDirectoryResponse,
                requested_dir: &str,
            ) -> Result<crate::repository::nitro::NitroFileResponse, InternalError> {
                match self {
                    $($v::$name(handler) => handler.process_storage_files(directory, requested_dir).await,)*
                    _ => unsafe{ std::hint::unreachable_unchecked() },

                }
            }
            async fn post_deploy(
                &self,
                project_folder: String,
                version_folder: String,
                model: crate::system::user::database::Model,
                version_data: crate::repository::nitro::VersionData,
            ) -> Result<(), InternalError> {
                match self {
                    $($v::$name(handler) => handler.post_deploy(project_folder, version_folder, model, version_data).await,)*
                    _ => unsafe{ std::hint::unreachable_unchecked() },

                }
            }
        }
    };
}
pub(crate) use nitro_repo_handler;

/// Impls the NitroRepository for the DynamicNitroRepositoryHandler
/// # Arguments
/// Array<Name> of the Repository Types
macro_rules! nitro_repo_handler {
    ($v:ident, $($name: ident, $ty:tt),*) => {
        #[async_trait::async_trait]
        impl<'a, StorageType: Storage> NitroRepositoryHandler<StorageType>
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
                _project: &str,
            ) -> Result<Option<crate::repository::nitro::NitroRepoVersions>, InternalError> {
                match self {
                    $($v::$name(handler) => handler.get_versions(project).await,)*
                    _ => unsafe{ std::hint::unreachable_unchecked() },

                }
            }

            async fn get_project_specific_version(
                &self,
                _project: &str,
                _version: &str,
            ) -> Result<Option<crate::repository::response::Project>, InternalError> {
                match self {
                    $($v::$name(handler) => handler.get_project_specific_version(project, version).await,)*
                    _ => unsafe{ std::hint::unreachable_unchecked() },
                }
            }
            async fn get_project_latest(
                &self,
                _project: &str,
            ) -> Result<Option<crate::repository::response::Project>, InternalError> {
                match self {
                    $($v::$name(handler) => handler.get_project_latest(project).await,)*
                    _ => unsafe{ std::hint::unreachable_unchecked() },

                }
            }
            async fn latest_version(&self, _project: &str) -> Result<Option<String>, InternalError> {
                match self {
                    $($v::$name(handler) => handler.latest_version(project).await,)*
                    _ => unsafe{ std::hint::unreachable_unchecked() },

                }
            }
            async fn process_storage_files(
                &self,
                _directory: crate::storage::file::StorageDirectoryResponse,
                _requested_dir: &str,
            ) -> Result<crate::repository::nitro::NitroFileResponse, InternalError> {
                match self {
                    $($v::$name(handler) => handler.process_storage_files(directory, requested_dir).await,)*
                    _ => unsafe{ std::hint::unreachable_unchecked() },

                }
            }
            async fn post_deploy(
                &self,
                _project_folder: String,
                _version_folder: String,
                _model: crate::system::user::database::Model,
                _version_data: crate::repository::nitro::VersionData,
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

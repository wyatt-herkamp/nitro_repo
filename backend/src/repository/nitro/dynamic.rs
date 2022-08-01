/// Impls the NitroRepository for the DynamicNitroRepositoryHandler
/// # Arguments
/// Array<Name> of the Repository Types
macro_rules! nitro_repo_handler {
    ($v:ident, $($name: ident, $ty:tt),*) => {
        // Configs that only nitro repositories can have
        crate::repository::handler::repository_config_group!($v, crate::repository::settings::badge::BadgeSettings, $($name),*);
        crate::repository::handler::repository_config_group!($v, crate::repository::settings::frontend::Frontend, $($name),*);

        #[async_trait::async_trait]
        impl<'a, StorageType: Storage> crate::repository::nitro::nitro_repository::NitroRepositoryHandler<StorageType>
            for $v<StorageType>
        {
            fn parse_project_to_directory<S: Into<String>>(_value: S) -> String {
                unsafe{ std::hint::unreachable_unchecked() }
            }
            #[inline(always)]
            fn supports_nitro(&self) -> bool {
                match self {
                    $($v::$name(handler) => handler.supports_nitro(),)*
                    _ => false,
                }
            }
            async fn get_repository_listing(&self) -> Result<Option<crate::repository::nitro::RepositoryListing>, InternalError> {
                match self {
                    $($v::$name(handler) => handler.get_repository_listing().await,)*
                    _ => unsafe{ std::hint::unreachable_unchecked() },
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
                version: &str,        generator_cache: Arc<crate::generators::GeneratorCache>,

            ) -> Result<Option<crate::repository::response::Project>, InternalError> {
                match self {
                    $($v::$name(handler) => handler.get_project_specific_version(project, version, generator_cache).await,)*
                    _ => unsafe{ std::hint::unreachable_unchecked() },
                }
            }
            async fn get_project_latest(
                &self,
                project: &str,        generator_cache: Arc<crate::generators::GeneratorCache>,

            ) -> Result<Option<crate::repository::response::Project>, InternalError> {
                match self {
                    $($v::$name(handler) => handler.get_project_latest(project, generator_cache).await,)*
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
                model: crate::system::user::database::UserSafeData,
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

macro_rules! main_nitro_handler {
    ($v:ident, $($name: ident, $ty:tt),*) => {
        crate::repository::nitro::dynamic::nitro_repo_handler!($v, $($name, $ty),*);
        pub mod nitro_configs{
            crate::repository::web::multi::configs::define_repository_config_handlers_group!("badge", crate::repository::settings::badge::BadgeSettings, $($name),*);
            crate::repository::web::multi::configs::define_repository_config_handlers_group!("frontend", crate::repository::settings::frontend::Frontend, $($name),*);
        }

    };
}
pub(crate) use main_nitro_handler;

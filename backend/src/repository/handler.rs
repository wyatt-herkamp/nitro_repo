use actix_web::http::header::HeaderMap;
use actix_web::http::StatusCode;
use actix_web::web::Bytes;
use actix_web::Error;
use async_trait::async_trait;
use sea_orm::DatabaseConnection;
use std::sync::Arc;

use crate::authentication::Authentication;
use crate::error::api_error::APIError;
use crate::error::internal_error::InternalError;

use crate::repository::response::RepoResponse;
use crate::repository::settings::RepositoryConfig;
use crate::storage::models::Storage;
use serde::{Deserialize, Serialize};
#[async_trait]
pub trait Repository<S: Storage>: Send + Sync + Clone {
    fn get_repository(&self) -> &RepositoryConfig;
    fn get_mut_config(&mut self) -> &mut RepositoryConfig;
    fn get_storage(&self) -> &S;

    /// Handles a get request to a Repo
    async fn handle_get(
        &self,
        _path: &str,
        _http: &HeaderMap,
        _conn: &DatabaseConnection,
        _authentication: Authentication,
    ) -> Result<RepoResponse, actix_web::Error> {
        Err(APIError::from((
            "Get is not implemented for this type",
            StatusCode::IM_A_TEAPOT,
        ))
        .into())
    }
    /// Handles a Post Request to a Repo
    async fn handle_post(
        &self,
        _path: &str,
        _http: &HeaderMap,
        _conn: &DatabaseConnection,
        _authentication: Authentication,
        _bytes: Bytes,
    ) -> Result<RepoResponse, actix_web::Error> {
        Err(APIError::from((
            "POST is not implemented for this type",
            StatusCode::IM_A_TEAPOT,
        ))
        .into())
    }
    /// Handles a PUT Request to a Repo
    async fn handle_put(
        &self,
        _path: &str,
        _http: &HeaderMap,
        _conn: &DatabaseConnection,
        _authentication: Authentication,
        _bytes: Bytes,
    ) -> Result<RepoResponse, actix_web::Error> {
        Err(APIError::from((
            "PUT is not implemented for this type",
            StatusCode::IM_A_TEAPOT,
        ))
        .into())
    }
    /// Handles a PATCH Request to a Repo
    async fn handle_patch(
        &self,
        _path: &str,
        _http: &HeaderMap,
        _conn: &DatabaseConnection,
        _authentication: Authentication,
        _bytes: Bytes,
    ) -> Result<RepoResponse, actix_web::Error> {
        Err(APIError::from((
            "Patch is not implemented for this type",
            StatusCode::IM_A_TEAPOT,
        ))
        .into())
    }
    /// Handles a HAPIResponseAD Request to a Repo
    async fn handle_head(
        &self,
        _path: &str,
        _http: &HeaderMap,
        _conn: &DatabaseConnection,
        _authentication: Authentication,
    ) -> Result<RepoResponse, actix_web::Error> {
        Err(APIError::from((
            "Head is not implemented for this type",
            StatusCode::IM_A_TEAPOT,
        ))
        .into())
    }
}

macro_rules! repository_handler {
    ($name:ident, $($repository_type:ident,$repository_tt:tt),*) => {
        #[derive(Debug)]
        pub enum $name<StorageType: Storage> {
            $(
                $repository_type($repository_tt<StorageType>),
            )*
        }
        impl< StorageType: Storage> Clone for $name<StorageType> {
            fn clone(&self) -> Self {
                match self {
                    $(
                        $name::$repository_type(repo) => $name::$repository_type((repo).clone()),
                    )*
                }
            }
        }
        // Implement From<$repository_tt> for $name
        $(
        impl<'a, StorageType: Storage> From<$repository_tt<StorageType>> for $name<StorageType> {
            fn from(repository: $repository_tt<StorageType>) -> Self {
                $name::$repository_type(repository)
            }
        }
        )*
        #[async_trait]
        impl<StorageType: Storage> Repository<StorageType>
            for $name<StorageType>
        {

            fn get_repository(&self) -> &RepositoryConfig {
                match self {
                    $(
                        $name::$repository_type(repository) => repository.get_repository(),
                    )*
                }
            }
            fn get_mut_config(&mut self) -> &mut RepositoryConfig {
                match self {
                    $(
                        $name::$repository_type(repository) => repository.get_mut_config(),
                    )*
                }
            }
            fn get_storage(&self) -> &StorageType {
                match self {
                    $(
                        $name::$repository_type(repository) => repository.get_storage(),
                    )*
                }
            }

            async fn handle_get(
                &self,
                path: &str,
                header: &HeaderMap,
                conn: &DatabaseConnection,
                authentication: Authentication,
            ) -> Result<RepoResponse, Error> {
                match self {
                    $(
                        $name::$repository_type(handler) => handler.handle_get(
                            path,
                            header,
                            conn,
                            authentication,
                        ).await,
                    )*
                }
            }
            async fn handle_post(
                &self,
                path: &str,
                header: &HeaderMap,
                conn: &DatabaseConnection,
                authentication: Authentication,
                bytes: Bytes,
            ) -> Result<RepoResponse, Error> {
                match self {
                    $(
                        $name::$repository_type(handler) => handler.handle_post(
                            path,
                            header,
                            conn,
                            authentication,
                            bytes,
                        ).await,
                    )*
                }
            }
            async fn handle_put(
                &self,
                path: &str,
                header: &HeaderMap,
                conn: &DatabaseConnection,
                authentication: Authentication,
                bytes: Bytes,
            ) -> Result<RepoResponse, Error> {
                match self {
                    $(
                        $name::$repository_type(handler) => handler.handle_put(
                            path,
                            header,
                            conn,
                            authentication,
                            bytes,
                        ).await,
                    )*
                }
            }
            async fn handle_patch(
                &self,
                path: &str,
                header: &HeaderMap,
                conn: &DatabaseConnection,
                authentication: Authentication,
                bytes: Bytes,
            ) -> Result<RepoResponse, Error> {
                match self {
                    $(
                        $name::$repository_type(handler) => handler.handle_patch(
                            path,
                            header,
                            conn,
                            authentication,
                            bytes,
                        ).await,
                    )*
                }
            }
            async fn handle_head(
                &self,
                path: &str,
                header: &HeaderMap,
                conn: &DatabaseConnection,
                authentication: Authentication,
            ) -> Result<RepoResponse, Error> {
                match self {
                    $(
                        $name::$repository_type(handler) => handler.handle_head(
                            path,
                            header,
                            conn,
                            authentication,
                        ).await,
                    )*
                }
            }
        }
        }

    }
pub(crate) use repository_handler;
/// Creates a DynamicRepositoryHandler to handle all types of Repositories
/// # Arguments
/// Array<Name, RepositoryHandlerType>
macro_rules! dynamic_repository_handler {
    ($($repository_type:ident,$repository_tt:tt),*) => {
        repository_handler!(
            DynamicRepositoryHandler,
            $($repository_type,$repository_tt),*
        );

        /// Types of Repositories that can exist.
        #[derive(Serialize, Deserialize, Clone, Debug, strum_macros::Display, strum_macros::EnumString)]
        pub enum RepositoryType {
            $($repository_type),*
        }

        impl<StorageType: Storage> DynamicRepositoryHandler<StorageType>{
                    #[inline]
        pub async fn new_dyn_storage(
            storage: Arc<StorageType>,
            repository_config: RepositoryConfig,
        ) -> Result<DynamicRepositoryHandler<StorageType>, InternalError> {
            let repository_handler = match repository_config.repository_type {
                $(
                    RepositoryType::$repository_type => {
                        let repository = $repository_tt::create(
                            repository_config,
                            storage,
                        ).await?;
                        DynamicRepositoryHandler::$repository_type(repository)
                    }
                )*
            };
            Ok(repository_handler)
        }
        }


    };
}
//    NPM,
//     NPMHandler,
//     Docker,
//     DockerHandler,
//     CI,
//     CIHandler,
//     Raw,
//     RawHandler

use crate::repository::maven::MavenHandler;
dynamic_repository_handler!(Maven, MavenHandler);
use crate::repository::nitro::nitro_repository::NitroRepositoryHandler;
crate::repository::nitro::dynamic::nitro_repo_handler!(DynamicRepositoryHandler,);

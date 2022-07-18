use actix_web::http::header::HeaderMap;
use actix_web::http::StatusCode;
use actix_web::web::Bytes;
use actix_web::Error;
use async_trait::async_trait;
use sea_orm::DatabaseConnection;
use tokio::sync::RwLockReadGuard;

use crate::authentication::Authentication;
use crate::error::api_error::APIError;
use crate::error::internal_error::InternalError;
use crate::repository::ci::CIHandler;
use crate::repository::docker::DockerHandler;
use crate::repository::maven::MavenHandler;
use crate::repository::npm::NPMHandler;
use crate::repository::raw::RawHandler;
use crate::repository::response::RepoResponse;
use crate::repository::settings::RepositoryConfig;
use crate::repository::settings::RepositoryType;
use crate::storage::models::Storage;

#[async_trait]
pub trait RepositoryHandler<'a, S: Storage>: Send + Sync {
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
/// Creates a DynamicRepositoryHandler to handle all types of Repositories
/// # Arguments
/// Array<Name, RepositoryHandlerType>
macro_rules! repository_handler {
    ($($repository_type:ident,$repository_tt:tt),*) => {
        pub enum DynamicRepositoryHandler<'a, StorageType: Storage> {
            $(
                $repository_type($repository_tt<'a, StorageType>),
            )*
        }
        #[inline]
        pub async fn get_repository_handler<StorageType: Storage>(
            storage: RwLockReadGuard<'_, StorageType>,
            repository_config: RepositoryConfig,
        ) -> Result<Option<DynamicRepositoryHandler<StorageType>>, InternalError> {
            match repository_config.repository_type {
                $(
                    RepositoryType::$repository_type => {
                        let handler = $repository_tt::create(repository_config, storage);
                        Ok(Some(DynamicRepositoryHandler::$repository_type(handler)))
                    },
                )*
            }
        }

        #[async_trait]
        impl<'a, StorageType: Storage> RepositoryHandler<'a, StorageType>
            for DynamicRepositoryHandler<'a, StorageType>
        {
            async fn handle_get(
                &self,
                path: &str,
                header: &HeaderMap,
                conn: &DatabaseConnection,
                authentication: Authentication,
            ) -> Result<RepoResponse, Error> {
                match self {
                    $(
                        DynamicRepositoryHandler::$repository_type(handler) => handler.handle_get(
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
                        DynamicRepositoryHandler::$repository_type(handler) => handler.handle_post(
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
                        DynamicRepositoryHandler::$repository_type(handler) => handler.handle_put(
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
                        DynamicRepositoryHandler::$repository_type(handler) => handler.handle_patch(
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
                        DynamicRepositoryHandler::$repository_type(handler) => handler.handle_head(
                            path,
                            header,
                            conn,
                            authentication,
                        ).await,
                    )*
                }
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

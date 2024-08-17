#![allow(unused_variables)]

use std::future::Future;

use axum::{
    body::Body,
    response::{IntoResponse, Response},
};

use ::http::StatusCode;

use nr_macros::DynRepositoryHandler;
use nr_storage::DynStorage;
mod staging;
pub use staging::*;
mod repo_http;
pub use repo_http::*;
pub mod maven;
mod repo_type;
pub use repo_type::*;
use thiserror::Error;
use uuid::Uuid;

use crate::error::BadRequestErrors;
pub trait Repository: Send + Sync + Clone {
    fn get_storage(&self) -> DynStorage;
    /// The Repository type. This is used to identify the Repository type in the database
    fn get_type(&self) -> &'static str;
    /// Config types that this Repository type has.
    fn config_types(&self) -> Vec<&str>;
    fn name(&self) -> String;
    fn id(&self) -> Uuid;
    fn is_active(&self) -> bool;

    async fn reload(&self) -> Result<(), RepositoryFactoryError> {
        Ok(())
    }
    /// Handles a get request to a Repo
    fn handle_get(
        &self,
        request: RepositoryRequest,
    ) -> impl Future<Output = Result<RepoResponse, RepositoryHandlerError>> + Send {
        async {
            Ok(RepoResponse::unsupported_method_response(
                request.parts.method,
                self.get_type(),
            ))
        }
    }
    /// Handles a Post Request to a Repo
    fn handle_post(
        &self,
        request: RepositoryRequest,
    ) -> impl Future<Output = Result<RepoResponse, RepositoryHandlerError>> + Send {
        async {
            Ok(RepoResponse::unsupported_method_response(
                request.parts.method,
                self.get_type(),
            ))
        }
    }
    /// Handles a PUT Request to a Repo
    fn handle_put(
        &self,
        request: RepositoryRequest,
    ) -> impl Future<Output = Result<RepoResponse, RepositoryHandlerError>> + Send {
        async {
            Ok(RepoResponse::unsupported_method_response(
                request.parts.method,
                self.get_type(),
            ))
        }
    }
    /// Handles a PATCH Request to a Repo
    fn handle_patch(
        &self,
        request: RepositoryRequest,
    ) -> impl Future<Output = Result<RepoResponse, RepositoryHandlerError>> + Send {
        async {
            Ok(RepoResponse::unsupported_method_response(
                request.parts.method,
                self.get_type(),
            ))
        }
    }
    fn handle_delete(
        &self,
        request: RepositoryRequest,
    ) -> impl Future<Output = Result<RepoResponse, RepositoryHandlerError>> + Send {
        async {
            Ok(RepoResponse::unsupported_method_response(
                request.parts.method,
                self.get_type(),
            ))
        }
    }
    /// Handles a HAPIResponseAD Request to a Repo
    fn handle_head(
        &self,
        request: RepositoryRequest,
    ) -> impl Future<Output = Result<RepoResponse, RepositoryHandlerError>> + Send {
        async {
            Ok(RepoResponse::unsupported_method_response(
                request.parts.method,
                self.get_type(),
            ))
        }
    }
    fn handle_other(
        &self,
        request: RepositoryRequest,
    ) -> impl Future<Output = Result<RepoResponse, RepositoryHandlerError>> + Send {
        async {
            Ok(RepoResponse::unsupported_method_response(
                request.parts.method,
                self.get_type(),
            ))
        }
    }
}

#[derive(Debug, Clone, DynRepositoryHandler)]
pub enum DynRepository {
    Maven(maven::MavenRepository),
}
#[derive(Debug, Error)]
pub enum RepositoryHandlerError {
    #[error("Database Error: {0}")]
    SQLXError(#[from] sqlx::Error),
    #[error("Storage Error: {0}")]
    StorageError(#[from] nr_storage::StorageError),
    #[error("Unexpected Missing Body")]
    MissingBody,
    #[error("Invalid JSON: {0}")]
    InvalidJson(#[from] serde_json::Error),
    #[error("Bad Request: {0}")]
    BadRequest(#[from] BadRequestErrors),
    #[error("Maven Repository Error: {0}")]
    MavenError(#[from] maven::MavenError),
    #[error("IO Error: {0}")]
    IOError(#[from] std::io::Error),
}
impl IntoResponse for RepositoryHandlerError {
    fn into_response(self) -> Response {
        match self {
            RepositoryHandlerError::StorageError(error) => Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(Body::from(format!(
                    "Error from Internal Storage System. Please contact your admin \n {}",
                    error
                )))
                .unwrap(),
            RepositoryHandlerError::MavenError(error) => error.into_response(),
            RepositoryHandlerError::BadRequest(error) => error.into_response(),
            other => Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(Body::from(format!(
                    "Internal Service Error  Please contact your admin \n {}",
                    other
                )))
                .unwrap(),
        }
    }
}

use crate::authentication::Authentication;
use actix_web::http::header::HeaderMap;
use actix_web::web::Bytes;
use std::sync::Arc;

use async_trait::async_trait;
use sea_orm::DatabaseConnection;

use crate::repository::data::{RepositoryConfig, RepositorySetting, RepositoryType};
use crate::repository::error::RepositoryError;
use crate::repository::error::RepositoryError::InternalError;
use crate::repository::maven::models::MavenSettings;
use crate::repository::maven::MavenHandler;

use crate::repository::npm::models::NPMSettings;
use crate::repository::npm::NPMHandler;
use crate::repository::response::RepoResponse;
use crate::storage::models::Storage;

#[async_trait]
pub trait RepositoryHandler<'a> {
    /// Handles a get request to a Repo
    async fn handle_get(
        &self,
        _path: &str,
        _http: &HeaderMap,
        _conn: &DatabaseConnection,
        _authentication: Authentication,
    ) -> Result<RepoResponse, crate::repository::error::RepositoryError> {
        Ok(RepoResponse::IAmATeapot(
            "Get is not implemented for this type".to_string(),
        ))
    }
    /// Handles a Post Request to a Repo
    async fn handle_post(
        &self,
        _path: &str,
        _http: &HeaderMap,
        _conn: &DatabaseConnection,
        _authentication: Authentication,
        _bytes: Bytes,
    ) -> Result<RepoResponse, crate::repository::error::RepositoryError> {
        Ok(RepoResponse::IAmATeapot(
            "Get is not implemented for this type".to_string(),
        ))
    }
    /// Handles a PUT Request to a Repo
    async fn handle_put(
        &self,
        _path: &str,
        _http: &HeaderMap,
        _conn: &DatabaseConnection,
        _authentication: Authentication,
        _bytes: Bytes,
    ) -> Result<RepoResponse, crate::repository::error::RepositoryError> {
        Ok(RepoResponse::IAmATeapot(
            "Get is not implemented for this type".to_string(),
        ))
    }
    /// Handles a PATCH Request to a Repo
    async fn handle_patch(
        &self,
        _path: &str,
        _http: &HeaderMap,
        _conn: &DatabaseConnection,
        _authentication: Authentication,
        _bytes: Bytes,
    ) -> Result<RepoResponse, crate::repository::error::RepositoryError> {
        Ok(RepoResponse::IAmATeapot(
            "Get is not implemented for this type".to_string(),
        ))
    }
    /// Handles a Hcrate::repository::error::RepositoryErrorAD Request to a Repo
    async fn handle_head(
        &self,
        _path: &str,
        _http: &HeaderMap,
        _conn: &DatabaseConnection,
        _authentication: Authentication,
    ) -> Result<RepoResponse, crate::repository::error::RepositoryError> {
        Ok(RepoResponse::IAmATeapot(
            "Get is not implemented for this type".to_string(),
        ))
    }
}

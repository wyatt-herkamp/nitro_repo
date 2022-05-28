use crate::authentication::Authentication;
use actix_web::http::header::HeaderMap;
use actix_web::http::StatusCode;
use actix_web::web::Bytes;

use crate::error::api_error::APIError;
use async_trait::async_trait;
use sea_orm::DatabaseConnection;

use crate::repository::response::RepoResponse;

#[async_trait]
pub trait RepositoryHandler<'a>: Send + Sync {
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

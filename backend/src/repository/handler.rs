use crate::authentication::Authentication;
use actix_web::http::header::HeaderMap;
use actix_web::web::Bytes;
use actix_web::HttpRequest;
use async_trait::async_trait;
use sea_orm::DatabaseConnection;

use crate::repository::data::{RepositoryConfig, RepositorySetting};
use crate::repository::error::RepositoryError;
use crate::repository::maven::models::MavenSettings;
use crate::repository::npm::models::NPMSettings;
use crate::repository::response::RepoResponse;
use crate::storage::models::Storage;

pub enum Repository {
    Maven(RepositoryConfig<MavenSettings>),
    NPM(RepositoryConfig<NPMSettings>),
}

impl Repository {
    pub async fn load(
        _storage: &Storage,
        _name: &str,
    ) -> Result<Option<Repository>, RepositoryError> {
        todo!();
    }
    async fn handle_get(
        &self,
        _storage: &Storage,
        _path: &str,
        _http: &HttpRequest,
        _conn: &DatabaseConnection,
    ) -> Result<RepoResponse, crate::repository::error::RepositoryError> {
        todo!()
    }

    async fn handle_post(
        &self,
        _storage: &Storage,
        _path: &str,
        _http: &HttpRequest,
        _conn: &DatabaseConnection,
        _bytes: Bytes,
    ) -> Result<RepoResponse, crate::repository::error::RepositoryError> {
        todo!()
    }

    async fn handle_put(
        &self,
        _storage: &Storage,
        _path: &str,
        _http: &HttpRequest,
        _conn: &DatabaseConnection,
        _bytes: Bytes,
    ) -> Result<RepoResponse, crate::repository::error::RepositoryError> {
        todo!()
    }

    async fn handle_patch(
        &self,
        _storage: &Storage,
        _path: &str,
        _http: &HttpRequest,
        _conn: &DatabaseConnection,
        _bytes: Bytes,
    ) -> Result<RepoResponse, crate::repository::error::RepositoryError> {
        todo!()
    }

    async fn handle_head(
        &self,
        _storage: &Storage,
        _path: &str,
        _http: &HttpRequest,
        _conn: &DatabaseConnection,
    ) -> Result<RepoResponse, crate::repository::error::RepositoryError> {
        todo!()
    }
}

#[async_trait]
pub trait RepositoryHandler<T: RepositorySetting> {
    /// Handles a get request to a Repo
    async fn handle_get(
        _repository: &RepositoryConfig<T>,
        _storage: &Storage,
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
        _repository: &RepositoryConfig<T>,
        _storage: &Storage,
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
        _repository: &RepositoryConfig<T>,
        _storage: &Storage,
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
        _repository: &RepositoryConfig<T>,
        _storage: &Storage,
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
        _repository: &RepositoryConfig<T>,
        _storage: &Storage,
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

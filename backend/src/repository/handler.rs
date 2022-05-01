use std::error::Error;
use std::rc::Rc;
use actix_web::http::header::HeaderMap;
use actix_web::HttpRequest;
use actix_web::web::Bytes;
use async_trait::async_trait;
use sea_orm::DatabaseConnection;
use crate::authentication::Authentication;
use crate::error::internal_error::InternalError;
use crate::repository;
use crate::repository::maven::models::MavenSettings;
use crate::repository::data::{RepositoryConfig, RepositorySetting};
use crate::repository::error::RepositoryError;
use crate::repository::npm::models::NPMSettings;
use crate::repository::response::{RepoResponse};
use crate::storage::models::Storage;

pub enum Repository {
    Maven(RepositoryConfig<MavenSettings>),
    NPM(RepositoryConfig<NPMSettings>),
}


impl Repository {
    pub async fn load(storage: &Storage, name: &str)->Result<Option<Repository>,RepositoryError>{
        todo!();
    }
    async fn handle_get(&self, storage: &Storage, path: &str, http: &HttpRequest, conn: &DatabaseConnection) -> Result<RepoResponse,crate::repository::error::RepositoryError> {
        todo!()
    }

    async fn handle_post(&self, storage: &Storage, path: &str, http: &HttpRequest, conn: &DatabaseConnection, bytes: Bytes) -> Result<RepoResponse, crate::repository::error::RepositoryError> {
        todo!()
    }

    async fn handle_put(&self, storage: &Storage, path: &str, http: &HttpRequest, conn: &DatabaseConnection, bytes: Bytes) -> Result<RepoResponse, crate::repository::error::RepositoryError> {
        todo!()
    }

    async fn handle_patch(&self, storage: &Storage, path: &str, http: &HttpRequest, conn: &DatabaseConnection, bytes: Bytes) -> Result<RepoResponse, crate::repository::error::RepositoryError> {
        todo!()
    }

    async fn handle_head(&self, storage: &Storage, path: &str, http: &HttpRequest, conn: &DatabaseConnection) -> Result<RepoResponse, crate::repository::error::RepositoryError> {
        todo!()
    }
}

#[async_trait]
pub trait RepositoryHandler<T: RepositorySetting> {
    /// Handles a get request to a Repo
    async fn handle_get(
        repository: &RepositoryConfig<T>,
        storage: &Storage,
        path: &str,
        http: &HeaderMap,
        conn: &DatabaseConnection,
        authentication: Authentication,
    ) -> Result<RepoResponse, crate::repository::error::RepositoryError> { Ok(RepoResponse::IAmATeapot("Get is not implemented for this type".to_string())) }
    /// Handles a Post Request to a Repo
    async fn handle_post(
        repository: &RepositoryConfig<T>,
        storage: &Storage,
        path: &str,
        http: &HeaderMap,
        conn: &DatabaseConnection, authentication: Authentication,
        bytes: Bytes,
    ) -> Result<RepoResponse, crate::repository::error::RepositoryError> { Ok(RepoResponse::IAmATeapot("Get is not implemented for this type".to_string())) }
    /// Handles a PUT Request to a Repo
    async fn handle_put(
        repository: &RepositoryConfig<T>,
        storage: &Storage,
        path: &str,
        http: &HeaderMap,
        conn: &DatabaseConnection, authentication: Authentication,
        bytes: Bytes,
    ) -> Result<RepoResponse, crate::repository::error::RepositoryError> { Ok(RepoResponse::IAmATeapot("Get is not implemented for this type".to_string())) }
    /// Handles a PATCH Request to a Repo
    async fn handle_patch(
        repository: &RepositoryConfig<T>,
        storage: &Storage,
        path: &str,
        http: &HeaderMap,
        conn: &DatabaseConnection, authentication: Authentication,
        bytes: Bytes,
    ) -> Result<RepoResponse, crate::repository::error::RepositoryError> { Ok(RepoResponse::IAmATeapot("Get is not implemented for this type".to_string())) }
    /// Handles a Hcrate::repository::error::RepositoryErrorAD Request to a Repo
    async fn handle_head(
        repository: &RepositoryConfig<T>,
        storage: &Storage,
        path: &str,
        http: &HeaderMap,
        conn: &DatabaseConnection, authentication: Authentication,
    ) -> Result<RepoResponse, crate::repository::error::RepositoryError> { Ok(RepoResponse::IAmATeapot("Get is not implemented for this type".to_string())) }
}
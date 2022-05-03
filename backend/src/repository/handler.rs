use crate::authentication::Authentication;
use actix_web::http::header::HeaderMap;
use actix_web::web::Bytes;

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

pub enum Repository {
    Maven(RepositoryConfig<MavenSettings>),
    NPM(RepositoryConfig<NPMSettings>),
}

impl Repository {
    pub async fn load(
        storage: &Storage,
        name: &str,
    ) -> Result<Option<Repository>, RepositoryError> {
        let repository_value = storage.get_repository_value(name).await?;
        if repository_value.is_none() {
            return Ok(None);
        }
        let repository_value = repository_value.unwrap();
        return match &repository_value.repository_type {
            RepositoryType::Maven => {
                let main = storage.get_repository::<MavenSettings>(name).await?;
                if main.is_none() {
                    return Err(InternalError(
                        "Repository Registered but not found".to_string(),
                    ));
                }
                Ok(Some(Repository::Maven(main.unwrap())))
            }
            RepositoryType::NPM => {
                let main = storage.get_repository::<NPMSettings>(name).await?;
                if main.is_none() {
                    return Err(InternalError(
                        "Repository Registered but not found".to_string(),
                    ));
                }

                Ok(Some(Repository::NPM(main.unwrap())))
            }
        };
    }
    async fn handle_get(
        &self,
        storage: &Storage,
        path: &str,
        headers: &HeaderMap,
        conn: &DatabaseConnection,
        auth: Authentication,
    ) -> Result<RepoResponse, crate::repository::error::RepositoryError> {
        match self {
            Repository::Maven(maven) => {
                MavenHandler::handle_get(maven, storage, path, headers, conn, auth).await
            }
            Repository::NPM(npm) => {
                NPMHandler::handle_get(npm, storage, path, headers, conn, auth).await
            }
        }
    }

    async fn handle_post(
        &self,
        storage: &Storage,
        path: &str,
        headers: &HeaderMap,
        conn: &DatabaseConnection,
        auth: Authentication,
        bytes: Bytes,
    ) -> Result<RepoResponse, crate::repository::error::RepositoryError> {
        match self {
            Repository::Maven(maven) => {
                MavenHandler::handle_post(maven, storage, path, headers, conn, auth, bytes).await
            }
            Repository::NPM(npm) => {
                NPMHandler::handle_post(npm, storage, path, headers, conn, auth, bytes).await
            }
        }
    }

    async fn handle_put(
        &self,
        storage: &Storage,
        path: &str,
        headers: &HeaderMap,
        conn: &DatabaseConnection,
        auth: Authentication,
        bytes: Bytes,
    ) -> Result<RepoResponse, crate::repository::error::RepositoryError> {
        match self {
            Repository::Maven(maven) => {
                MavenHandler::handle_put(maven, storage, path, headers, conn, auth, bytes).await
            }
            Repository::NPM(npm) => {
                NPMHandler::handle_put(npm, storage, path, headers, conn, auth, bytes).await
            }
        }
    }

    async fn handle_patch(
        &self,
        storage: &Storage,
        path: &str,
        headers: &HeaderMap,
        conn: &DatabaseConnection,
        auth: Authentication,
        bytes: Bytes,
    ) -> Result<RepoResponse, crate::repository::error::RepositoryError> {
        match self {
            Repository::Maven(maven) => {
                MavenHandler::handle_patch(maven, storage, path, headers, conn, auth, bytes).await
            }
            Repository::NPM(npm) => {
                NPMHandler::handle_patch(npm, storage, path, headers, conn, auth, bytes).await
            }
        }
    }

    async fn handle_head(
        &self,
        storage: &Storage,
        path: &str,
        headers: &HeaderMap,
        conn: &DatabaseConnection,
        auth: Authentication,
    ) -> Result<RepoResponse, crate::repository::error::RepositoryError> {
        match self {
            Repository::Maven(maven) => {
                MavenHandler::handle_head(maven, storage, path, headers, conn, auth).await
            }
            Repository::NPM(npm) => {
                NPMHandler::handle_head(npm, storage, path, headers, conn, auth).await
            }
        }
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

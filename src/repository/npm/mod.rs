use actix_web::HttpRequest;
use actix_web::web::Bytes;
use diesel::MysqlConnection;
use crate::error::internal_error::InternalError;
use crate::repository::repository::{RepoResponse, RepoResult, RepositoryRequest, RepositoryType};

pub struct NPMHandler;

impl RepositoryType for NPMHandler {
    fn handle_get(request: &RepositoryRequest, http: &HttpRequest,conn: &MysqlConnection) -> RepoResult {
        todo!()
    }

    fn handle_post(request: &RepositoryRequest,http: &HttpRequest, conn: &MysqlConnection, bytes: Bytes) -> RepoResult {
        todo!()
    }

    fn handle_put(request: &RepositoryRequest, http: &HttpRequest,conn: &MysqlConnection, bytes: Bytes) -> RepoResult {
        todo!()
    }

    fn handle_patch(request: &RepositoryRequest,http: &HttpRequest, conn: &MysqlConnection, bytes: Bytes) -> RepoResult {
        todo!()
    }

    fn handle_head(request: &RepositoryRequest,http: &HttpRequest, conn: &MysqlConnection) -> RepoResult {
        todo!()
    }

    fn handle_versions(request: &RepositoryRequest,http: &HttpRequest,conn: &MysqlConnection) -> RepoResult {
        todo!()
    }

    fn latest_version(request: &RepositoryRequest, http: &HttpRequest,conn: &MysqlConnection) -> Result<String, InternalError> {
        todo!()
    }
}
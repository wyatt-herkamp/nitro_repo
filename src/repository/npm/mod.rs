use actix_web::web::Bytes;
use diesel::MysqlConnection;
use crate::error::request_error::RequestError;
use crate::repository::repository::{RepoResponse, RepoResult, RepositoryRequest, RepositoryType};

pub struct NPMHandler;

impl RepositoryType for NPMHandler {
    fn handle_get(request: RepositoryRequest, conn: &MysqlConnection) -> RepoResult {
        todo!()
    }

    fn handle_post(request: RepositoryRequest, conn: &MysqlConnection, bytes: Bytes) -> RepoResult {
        todo!()
    }

    fn handle_put(request: RepositoryRequest, conn: &MysqlConnection, bytes: Bytes) -> RepoResult {
        todo!()
    }

    fn handle_patch(request: RepositoryRequest, conn: &MysqlConnection, bytes: Bytes) -> RepoResult {
        todo!()
    }

    fn handle_head(request: RepositoryRequest, conn: &MysqlConnection) -> RepoResult {
        todo!()
    }

    fn handle_versions(request: RepositoryRequest, conn: &MysqlConnection) -> RepoResult {
        todo!()
    }

    fn latest_version(request: RepositoryRequest, conn: &MysqlConnection) -> Result<String, RequestError> {
        todo!()
    }
}
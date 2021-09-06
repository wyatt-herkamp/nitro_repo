use crate::error::request_error::RequestError;
use crate::repository::models::Repository;


use crate::storage::models::Storage;
use actix_web::web::Bytes;
use actix_web::{HttpRequest};
use diesel::MysqlConnection;
use std::path::{PathBuf};

pub enum RepoResponse {
    FileList(Vec<String>),
    FileResponse(PathBuf),
    Ok,
    NotFound,
    NotAuthorized,
}

pub type RepoResult = Result<RepoResponse, RequestError>;

pub struct RepositoryRequest {
    pub request: HttpRequest,
    pub storage: Storage,
    pub repository: Repository,
    pub value: String,
}

pub trait RepositoryType {
    fn handle_get(request: RepositoryRequest, conn: &MysqlConnection) -> RepoResult;
    fn handle_post(request: RepositoryRequest, conn: &MysqlConnection, bytes: Bytes) -> RepoResult;
    fn handle_put(request: RepositoryRequest, conn: &MysqlConnection, bytes: Bytes) -> RepoResult;
    fn handle_patch(request: RepositoryRequest, conn: &MysqlConnection, bytes: Bytes)
        -> RepoResult;
    fn handle_head(request: RepositoryRequest, conn: &MysqlConnection) -> RepoResult;
}

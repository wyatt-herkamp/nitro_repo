use std::collections::HashMap;

use crate::repository::models::Repository;

use crate::storage::models::Storage;
use actix_web::web::Bytes;
use actix_web::HttpRequest;
use diesel::MysqlConnection;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use serde_json::Value;
use crate::error::internal_error::InternalError;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RepositoryFile {
    pub name: String,
    pub full_path: String,
    pub directory: bool,
    pub data: HashMap<String, Value>,
}


pub enum RepoResponse {
    FileList(Vec<RepositoryFile>),
    FileResponse(PathBuf),
    Ok,
    NotFound,
    NotAuthorized,
    BadRequest(String),
    IAmATeapot(String),
    VersionResponse(Vec<Version>),
}

pub type RepoResult = Result<RepoResponse, InternalError>;

pub struct RepositoryRequest {
    pub storage: Storage,
    pub repository: Repository,
    pub value: String,
}
impl RepositoryRequest{
    pub fn new(storage: Storage, repository: Repository, value: String)->RepositoryRequest{
        return RepositoryRequest {
            storage,
            repository,
            value
        }
    }
}
#[derive(Debug, Serialize, Deserialize)]
pub struct Version {
    pub version: String,
    pub artifacts: Vec<String>,
}

pub trait RepositoryType {
    fn handle_get(request: &RepositoryRequest,http:&HttpRequest, conn: &MysqlConnection) -> RepoResult;
    fn handle_post(request: &RepositoryRequest,http:&HttpRequest, conn: &MysqlConnection, bytes: Bytes) -> RepoResult;
    fn handle_put(request: &RepositoryRequest,http:&HttpRequest, conn: &MysqlConnection, bytes: Bytes) -> RepoResult;
    fn handle_patch(request: &RepositoryRequest,http:&HttpRequest, conn: &MysqlConnection, bytes: Bytes)
                    -> RepoResult;
    fn handle_head(request: &RepositoryRequest, http:&HttpRequest,conn: &MysqlConnection) -> RepoResult;
    fn handle_versions(request: &RepositoryRequest,http:&HttpRequest, conn: &MysqlConnection) -> RepoResult;

    fn latest_version(
        request: &RepositoryRequest,http:&HttpRequest,
        conn: &MysqlConnection,
    ) -> Result<String, InternalError>;
}

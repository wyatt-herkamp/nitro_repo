use std::collections::HashMap;
use std::path::PathBuf;

use actix_web::web::Bytes;
use actix_web::HttpRequest;
use diesel::MysqlConnection;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::error::internal_error::InternalError;
use crate::repository::frontend::FrontendResponse;
use crate::repository::models::{Repository, RepositorySummary};
use crate::storage::models::Storage;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RepositoryFile {
    pub name: String,
    pub full_path: String,
    pub directory: bool,
    pub data: HashMap<String, Value>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Project {
    pub repo_summary: RepositorySummary,
    pub versions: Vec<Version>,
    pub frontend_response: Option<FrontendResponse>,
}

/// Types of Valid Repo Responses
pub enum RepoResponse {
    FileList(Vec<RepositoryFile>),
    /// Responds all the information about the project
    ProjectResponse(Project),
    /// Respond a file so it can be downloaded
    FileResponse(PathBuf),
    /// Ok
    Ok,
    //Ok With Json
    OkWithJSON(String),
    /// CREATED WITH_JSON
    CreatedWithJSON(String),
    /// Not Found
    NotFound,
    /// Not Authorized
    NotAuthorized,
    /// Bad Request
    BadRequest(String),
    /// I am A Teapot. This is a joke. And is used inside Maven to state that Such as POST and PATCH
    IAmATeapot(String),
    /// A list of versions in a specific artifact. This is generated in Maven by bad code
    VersionListingResponse(Vec<Version>),
    VersionResponse(Version),
}

/// RepoResult
pub type RepoResult = Result<RepoResponse, InternalError>;

/// This is a Request to a Repository Handler
pub struct RepositoryRequest {
    /// The Storage that the Repo needs to be in
    pub storage: Storage,
    /// The Repository it needs to be in
    pub repository: Repository,
    /// Everything in the URL path after /storages/{STORAGE}/{REPOSITORY}
    pub value: String,
}

impl RepositoryRequest {
    pub fn new(storage: Storage, repository: Repository, value: String) -> RepositoryRequest {
        RepositoryRequest {
            storage,
            repository,
            value,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Version {
    pub version: String,
    #[serde(flatten)]
    pub other: HashMap<String, Value>,
}

pub trait RepositoryType {
    /// Handles a get request to a Repo
    fn handle_get(
        request: &RepositoryRequest,
        http: &HttpRequest,
        conn: &MysqlConnection,
    ) -> RepoResult;
    /// Handles a Post Request to a Repo
    fn handle_post(
        request: &RepositoryRequest,
        http: &HttpRequest,
        conn: &MysqlConnection,
        bytes: Bytes,
    ) -> RepoResult;
    /// Handles a PUT Request to a Repo
    fn handle_put(
        request: &RepositoryRequest,
        http: &HttpRequest,
        conn: &MysqlConnection,
        bytes: Bytes,
    ) -> RepoResult;
    /// Handles a PATCH Request to a Repo
    fn handle_patch(
        request: &RepositoryRequest,
        http: &HttpRequest,
        conn: &MysqlConnection,
        bytes: Bytes,
    ) -> RepoResult;
    /// Handles a HEAD Request to a Repo
    fn handle_head(
        request: &RepositoryRequest,
        http: &HttpRequest,
        conn: &MysqlConnection,
    ) -> RepoResult;
    /// Handles a List of versions request
    fn handle_versions(
        request: &RepositoryRequest,
        http: &HttpRequest,
        conn: &MysqlConnection,
    ) -> RepoResult;
    fn handle_version(
        request: &RepositoryRequest,
        http: &HttpRequest,
        conn: &MysqlConnection,
    ) -> RepoResult;
    fn handle_project(
        request: &RepositoryRequest,
        http: &HttpRequest,
        conn: &MysqlConnection,
    ) -> RepoResult;
    /// Returns the latest version published.
    fn latest_version(
        request: &RepositoryRequest,
        http: &HttpRequest,
        conn: &MysqlConnection,
    ) -> Result<String, InternalError>;
}

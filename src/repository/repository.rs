use std::collections::HashMap;
use crate::error::request_error::RequestError;
use crate::repository::models::Repository;

use crate::storage::models::Storage;
use actix_web::web::Bytes;
use actix_web::HttpRequest;
use diesel::MysqlConnection;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use serde_json::Value;

/// This is a response used in listing responses.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RepositoryFile {
    /// File name
    pub name: String,
    /// The full path to the file in reference to a URL Example format!("{}/{}/{}/{}", request.storage.name, request.repository.name, request.value, file);
    pub full_path: String,
    /// Is this a directory true or false
    pub directory: bool,
    /// Any other data. currently this in not implemented in frontend and can be ignored
    pub data: HashMap<String, Value>,
}

/// Types of Valid Repo Responses
pub enum RepoResponse {
    /// Directory Listings
    FileList(Vec<RepositoryFile>),
    /// Respond a file so it can be downloaded
    FileResponse(PathBuf),
    /// Ok
    Ok,
    /// Not Found
    NotFound,
    /// Not Authorized
    NotAuthorized,
    /// Bad Request
    BadRequest(String),
    /// I am A Teapot. This is a joke. And is used inside Maven to state that Such as POST and PATCH
    IAmATeapot(String),
    /// A list of versions in a specific artifact. This is generated in Maven by bad code
    VersionResponse(Vec<Version>),
}
/// RepoResult
pub type RepoResult = Result<RepoResponse, RequestError>;

/// This is a Request to a Repository Handler
pub struct RepositoryRequest {
    /// The original HttpRequest from Actix
    pub request: HttpRequest,
    /// The Storage that the Repo needs to be in
    pub storage: Storage,
    /// The Repository it needs to be in
    pub repository: Repository,
    /// Everything in the URL path after /storages/{STORAGE}/{REPOSITORY}
    pub value: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Version {
    pub version: String,
    pub artifacts: Vec<String>,
}

/// Implement this if you are adding a new Repository Type Handler
pub trait RepositoryType {
    /// Handles a get request to a Repo
    fn handle_get(request: RepositoryRequest, conn: &MysqlConnection) -> RepoResult;
    /// Handles a Post Request to a Repo
    fn handle_post(request: RepositoryRequest, conn: &MysqlConnection, bytes: Bytes) -> RepoResult;
    /// Handles a PUT Request to a Repo
    fn handle_put(request: RepositoryRequest, conn: &MysqlConnection, bytes: Bytes) -> RepoResult;
    /// Handles a PATCH Request to a Repo
    fn handle_patch(request: RepositoryRequest, conn: &MysqlConnection, bytes: Bytes)
                    -> RepoResult;
    /// Handles a HEAD Request to a Repo
    fn handle_head(request: RepositoryRequest, conn: &MysqlConnection) -> RepoResult;
    /// Handles a List of versions request
    fn handle_versions(request: RepositoryRequest, conn: &MysqlConnection) -> RepoResult;
    /// Returns the latest version published.
    fn latest_version(
        request: RepositoryRequest,
        conn: &MysqlConnection,
    ) -> Result<String, RequestError>;
}

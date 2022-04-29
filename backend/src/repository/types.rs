use std::collections::HashMap;

use actix_web::web::Bytes;
use actix_web::HttpRequest;
use sea_orm::DatabaseConnection;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::error::internal_error::InternalError;
use crate::repository::frontend::FrontendResponse;
use crate::repository::maven::models::MavenSettings;
use crate::repository::maven::MavenHandler;
use crate::repository::models::{Repository, RepositorySummary};
use crate::repository::nitro::{
    NitroFileResponse, NitroRepoVersions, NitroVersion, ProjectData, VersionData,
};
use crate::repository::npm::models::NPMSettings;
use crate::repository::npm::NPMHandler;
use crate::storage::models::{Storage, StorageFile};
use strum_macros::{Display, EnumString};
use crate::api_response::SiteResponse;
use crate::authentication::Authentication;
use crate::settings::models::StringMap;

//Requestable Data
pub type RDatabaseConnection = DatabaseConnection;
pub type RBytes = Bytes;

#[derive(Serialize, Deserialize, Clone, Debug, Display, EnumString)]
pub enum RepositoryType {
    Maven(MavenSettings),
    NPM(NPMSettings),
}

impl RepositoryType {
    pub async fn handle_get(
        &self,
        request: &RepositoryRequest,
        http: &HttpRequest,
        conn: &DatabaseConnection,
        auth: Authentication,
    ) -> RepoResult {
        match self {
            RepositoryType::Maven(_) => MavenHandler::handle_get(request, http, conn, auth).await,
            RepositoryType::NPM(_) => NPMHandler::handle_get(request, http, conn, auth).await,
        }
    }

    pub async fn handle_post(
        &self,
        _request: &RepositoryRequest,
        _http: &HttpRequest,
        _conn: &DatabaseConnection,
        _bytes: Bytes,
    ) -> RepoResult {
        match self {
            t => {
                return Ok(RepoResponse::IAmATeapot(format!(
                    "{} doesn't support this type of request",
                    t
                )));
            }
        }
    }

    pub async fn handle_put(
        &self,
        request: &RepositoryRequest,
        http: &HttpRequest,
        conn: &DatabaseConnection,
        bytes: Bytes, auth: Authentication,
    ) -> RepoResult {
        match self {
            RepositoryType::Maven(_) => MavenHandler::handle_put(request, http, conn, bytes, auth).await,
            RepositoryType::NPM(_) => NPMHandler::handle_put(request, http, conn, bytes, auth).await,
        }
    }

    pub async fn handle_patch(
        &self,
        _request: &RepositoryRequest,
        _http: &HttpRequest,
        _conn: &DatabaseConnection,
        _bytes: Bytes,
    ) -> RepoResult {
        match self {
            t => {
                return Ok(RepoResponse::IAmATeapot(format!(
                    "{} doesn't support this type of request",
                    t
                )));
            }
        }
    }

    pub async fn handle_head(
        &self,
        _request: &RepositoryRequest,
        _http: &HttpRequest,
        _conn: &DatabaseConnection,
    ) -> RepoResult {
        match self {
            t => {
                return Ok(RepoResponse::IAmATeapot(format!(
                    "{} doesn't support this type of request",
                    t
                )));
            }
        }
    }

    pub async fn handle_versions(
        &self,
        request: &RepositoryRequest,
        http: &HttpRequest,
        conn: &DatabaseConnection,
    ) -> RepoResult {
        match self {
            RepositoryType::Maven(_) => MavenHandler::handle_versions(request, http, conn).await,
            RepositoryType::NPM(_) => NPMHandler::handle_versions(request, http, conn).await,
        }
    }

    pub async fn handle_version(
        &self,
        request: &RepositoryRequest,
        version: String,
        http: &HttpRequest,
        conn: &DatabaseConnection,
    ) -> RepoResult {
        match self {
            RepositoryType::Maven(_) => {
                MavenHandler::handle_version(request, version, http, conn).await
            }
            RepositoryType::NPM(_) => {
                NPMHandler::handle_version(request, version, http, conn).await
            }
        }
    }

    pub async fn handle_project(
        &self,
        request: &RepositoryRequest,
        http: &HttpRequest,
        conn: &DatabaseConnection,
    ) -> RepoResult {
        match self {
            RepositoryType::Maven(_) => MavenHandler::handle_project(request, http, conn).await,
            RepositoryType::NPM(_) => NPMHandler::handle_project(request, http, conn).await,
        }
    }

    pub async fn latest_version(
        &self,
        request: &RepositoryRequest,
        http: &HttpRequest,
        conn: &DatabaseConnection,
    ) -> Result<Option<String>, InternalError> {
        match self {
            RepositoryType::Maven(_) => MavenHandler::latest_version(request, http, conn).await,
            RepositoryType::NPM(_) => NPMHandler::latest_version(request, http, conn).await,
        }
    }
}

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
    pub project: ProjectData,
    /// Version Data will be latest if not specified
    pub version: Option<VersionData>,
    pub frontend_response: Option<FrontendResponse>,
}

/// Types of Valid Repo Responses
pub enum RepoResponse {
    FileList(Vec<StorageFile>),
    NitroFileList(NitroFileResponse),
    /// Responds all the information about the project
    ProjectResponse(Project),
    /// Respond a file so it can be downloaded
    FileResponse(SiteResponse),
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
    VersionListingResponse(Vec<VersionResponse>),
    /// Classic Version Response will be removed
    NitroProjectResponse(ProjectData),
    NitroVersionListingResponse(NitroRepoVersions),
    NitroVersionResponse(VersionResponse),
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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VersionResponse {
    pub version: NitroVersion,
    #[serde(flatten)]
    pub other: HashMap<String, Value>,
}

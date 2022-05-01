use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::api_response::SiteResponse;

use crate::repository::frontend::FrontendResponse;

use crate::repository::nitro::{
    NitroFileResponse, NitroRepoVersions, NitroVersion, ProjectData, VersionData,
};

use crate::storage::models::StorageFile;

use crate::repository::data::RepositoryValue;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RepositoryFile {
    pub name: String,
    pub full_path: String,
    pub directory: bool,
    pub data: HashMap<String, Value>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Project {
    pub repo_summary: RepositoryValue,
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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VersionResponse {
    pub version: NitroVersion,
    #[serde(flatten)]
    pub other: HashMap<String, Value>,
}

use actix_web::body::BoxBody;
use actix_web::http::header::CONTENT_LOCATION;
use actix_web::http::StatusCode;
use actix_web::web::Json;
use actix_web::{HttpRequest, HttpResponse, Responder};
use std::collections::HashMap;

use crate::api_response::APIResponse;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::repository::data::RepositoryConfig;

use crate::repository::frontend::FrontendResponse;

use crate::repository::nitro::{
    NitroFileResponse, NitroRepoVersions, NitroVersion, ProjectData, VersionData,
};
use crate::storage::file::StorageFileResponse;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RepositoryFile {
    pub name: String,
    pub full_path: String,
    pub directory: bool,
    pub data: HashMap<String, Value>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Project {
    pub repo_summary: RepositoryConfig,
    pub project: ProjectData,
    /// Version Data will be latest if not specified
    pub version: Option<VersionData>,
    pub frontend_response: Option<FrontendResponse>,
}

/// Types of Valid Repo Responses
pub enum RepoResponse {
    Response(APIResponse),
    FileResponse(StorageFileResponse),
    HttpResponse(HttpResponse),
    Json(Value, StatusCode),
    PUTResponse(bool, String),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VersionResponse {
    pub version: NitroVersion,
    #[serde(flatten)]
    pub other: HashMap<String, Value>,
}
impl Responder for RepoResponse {
    type Body = BoxBody;

    fn respond_to(self, req: &HttpRequest) -> HttpResponse<Self::Body> {
        match self {
            RepoResponse::Response(response) => response.respond_to(req),
            RepoResponse::FileResponse(file) => file.respond_to(req),
            RepoResponse::HttpResponse(http) => http,
            RepoResponse::Json(value, status) => {
                Json(value).customize().with_status(status).respond_to(req)
            }
            RepoResponse::PUTResponse(exists, content_location) => {
                let header = (CONTENT_LOCATION, content_location);
                if exists {
                    HttpResponse::Created().insert_header(header)
                } else {
                    HttpResponse::NoContent().insert_header(header)
                }
            }
        }
    }
}

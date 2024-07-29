use actix_web::{
    http::{header::HeaderMap, StatusCode, Uri},
    FromRequest, HttpRequest, HttpResponse,
};
use derive_more::From;
use nr_storage::{StorageFile, StoragePath};
use serde_json::Value;

use crate::app::{authentication::RepositoryAuthentication, DatabaseConnection, NitroRepoData};
pub struct RepositoryRequest<'r> {
    pub headers: &'r HeaderMap,
    pub path: StoragePath,
    pub database_connection: DatabaseConnection,
    pub authentication: RepositoryAuthentication,
    pub nitro_repo: NitroRepoData,
    pub uri: &'r Uri,
}

#[derive(Debug, From)]
pub enum RepoResponse {
    FileResponse(StorageFile),
    HttpResponse(HttpResponse),
    Json(Value, StatusCode),
    PUTResponse(bool, String),
}
impl RepoResponse {
    pub fn new_from_string(value: impl Into<String>, status: StatusCode) -> Self {
        let value = value.into();
        let response = HttpResponse::build(status).body(value);
        RepoResponse::HttpResponse(response)
    }
}

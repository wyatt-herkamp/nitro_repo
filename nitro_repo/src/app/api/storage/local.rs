use axum::{
    Json,
    extract::State,
    response::{IntoResponse, Response},
    routing::post,
};
use nr_core::user::permissions::HasPermissions;
use serde::{Deserialize, Serialize};
use storage::NitroRepo;
use tracing::instrument;
use utoipa::{OpenApi, ToSchema};

use crate::{
    app::{api::storage, authentication::Authentication, responses::MissingPermission},
    error::InternalError,
    utils::ResponseBuilder,
};
#[derive(OpenApi)]
#[openapi(
    paths(path_helper),
    components(schemas(LocalStoragePathHelperRequest, LocalStoragePathHelperResponse,))
)]
pub struct LocalStorageAPI;
pub fn local_storage_routes() -> axum::Router<NitroRepo> {
    axum::Router::new().route("/path-helper", post(path_helper))
}
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct LocalStoragePathHelperRequest {
    pub path: Option<String>,
}
#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(tag = "type", content = "value")]
pub enum LocalStoragePathHelperResponse {
    /// The current working directory
    CurrentPath(String),
    /// A list of directories in the path
    Directories(Vec<String>),
    /// The path does not exist
    PathDoesNotExist,
}
#[utoipa::path(
    get,
    path = "/local/path-helper",
    responses(
        (status = 200, description = "a path suggestion", body = LocalStoragePathHelperResponse)
    )
)]
#[instrument]
pub async fn path_helper(
    auth: Authentication,
    State(site): State<NitroRepo>,
    Json(request): Json<LocalStoragePathHelperRequest>,
) -> Result<Response, InternalError> {
    if !auth.is_admin_or_system_manager() {
        return Ok(MissingPermission::StorageManager.into_response());
    }
    let path = request.path.unwrap_or_default().trim().to_owned();
    if path.is_empty() {
        let current_path = site
            .suggested_local_storage_path
            .to_string_lossy()
            .to_string();
        return Ok(
            ResponseBuilder::ok().json(&&LocalStoragePathHelperResponse::CurrentPath(current_path))
        );
    }
    let path = std::path::Path::new(&path);
    let response = if path.exists() {
        // List directories
        let mut directories = vec![];
        for entry in std::fs::read_dir(path).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.is_dir() {
                if let Some(file_name) = path.file_name() {
                    directories.push(file_name.to_string_lossy().to_string());
                }
            }
        }
        LocalStoragePathHelperResponse::Directories(directories)
    } else {
        LocalStoragePathHelperResponse::PathDoesNotExist
    };
    Ok(ResponseBuilder::ok().json(&response))
}

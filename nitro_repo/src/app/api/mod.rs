use axum::{extract::State, Json};
use http::StatusCode;
use nr_core::database::user::NewUserRequest;
use serde::{Deserialize, Serialize};
use tower_http::cors::CorsLayer;
use tracing::{error, instrument};
use utoipa::ToSchema;
pub mod repository;
pub mod storage;
pub mod user;
pub mod user_management;
use crate::error::InternalError;

use super::{authentication::password, Instance, NitroRepo, NitroRepoState};
pub fn api_routes() -> axum::Router<NitroRepo> {
    let router = axum::Router::new()
        .route("/info", axum::routing::get(info))
        .route("/install", axum::routing::post(install))
        .nest("/user", user::user_routes())
        .nest("/storage", storage::storage_routes())
        .nest(
            "/user-management",
            user_management::user_management_routes(),
        )
        .nest("/repository", repository::repository_routes())
        .layer(CorsLayer::very_permissive());

    router
}
#[utoipa::path(
    get,
    path = "/api/info",
    responses(
        (status = 200, description = "information about the Site", body = Instance)
    )
)]
#[instrument]
pub async fn info(State(site): NitroRepoState) -> Json<Instance> {
    let site = site.instance.lock().clone();
    Json(site)
}
#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct InstallRequest {
    pub user: NewUserRequest,
}
/// Installs the site with the first user. If Site is already installed, it will return a 404.
#[utoipa::path(
    post,
    request_body = InstallRequest,
    path = "/api/install",
    responses(
        (status = 204, description = "Site is now installed"),
        (status = 404, description = "Site is already installed"),
    )
)]
#[instrument]
pub async fn install(
    State(site): NitroRepoState,
    Json(request): Json<InstallRequest>,
) -> Result<StatusCode, InternalError> {
    {
        let instance = site.instance.lock();
        if instance.is_installed {
            return Ok(StatusCode::NOT_FOUND);
        }
    }
    let InstallRequest { mut user } = request;
    let password = user
        .password
        .as_ref()
        .and_then(|password| password::encrypt_password(password));
    if password.is_none() {
        error!("A Password must exist for the first user.");
        return Ok(StatusCode::BAD_REQUEST);
    }
    user.password = password;
    user.insert_admin(&site.database).await?;
    {
        let mut instance = site.instance.lock();
        instance.is_installed = true;
    }
    return Ok(StatusCode::NO_CONTENT);
}

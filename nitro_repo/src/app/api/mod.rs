use axum::{extract::State, response::IntoResponse, Json};
use http::StatusCode;
use nr_core::{database::user::NewUserRequest, user::permissions::UserPermissions};
use serde::{Deserialize, Serialize};
use tracing::{error, instrument};
pub mod repository;
pub mod storage;
pub mod user;
use crate::{error::internal_error::InternalError, utils::password::encrypt_password};

use super::{NitroRepo, NitroRepoState};
pub fn api_routes() -> axum::Router<NitroRepo> {
    axum::Router::new()
        .route("/api/info", axum::routing::get(info))
        .route("/api/install", axum::routing::post(install))
        .nest("/api/user", user::user_routes())
        .nest("/api/storage", storage::storage_routes())
        .nest("/api/repository", repository::repository_routes())
}
#[instrument]
pub async fn info(State(site): NitroRepoState) -> impl IntoResponse {
    let site = site.instance.lock().clone();
    Json(site)
}
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InstallRequest {
    pub user: NewUserRequest,
}
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
        .and_then(|password| encrypt_password(password));
    if password.is_none() {
        error!("A Password must exist for the first user.");
        return Ok(StatusCode::BAD_REQUEST);
    }
    user.password = password;
    user.insert(UserPermissions::admin(), &site.database)
        .await?;
    {
        let mut instance = site.instance.lock();
        instance.is_installed = true;
    }
    return Ok(StatusCode::OK);
}

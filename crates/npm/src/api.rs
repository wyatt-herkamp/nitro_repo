//! The browser half of npm's web login.
//!
//! `npm login` opens `/npm/login/{session}` in a browser. That page authenticates the user against
//! the site the normal way, shows them which repository is asking, and then calls
//! `POST /api/npm/login/{session}` here. This mints the repository token and hands it to the
//! waiting `npm` process through the `doneUrl` it is polling.
//!
//! See [`super::login::web_login`] for the registry side of the exchange.
//!
//! The session manager arrives as a request extension rather than off the application state, so
//! that it can stay owned by [`NpmRegistryType`](super::NpmRegistryType) and private to npm.
use std::sync::Arc;

use axum::{
    Extension, Router,
    extract::{Path, State},
    response::{IntoResponse, Response},
};
use nr_core::{
    database::entities::{repository::DBRepository, user::auth_token::NewRepositoryToken},
    user::permissions::{HasPermissions, RepositoryActions},
};
use nr_repository::SiteContext;
use nr_web_core::{
    authentication::Authentication, error::InternalError, responses::MissingPermission,
    utils::ResponseBuilder,
};
use serde::{Deserialize, Serialize};
use tracing::{info, instrument};
use utoipa::{OpenApi, ToSchema};
use uuid::Uuid;

use super::login::web_login::NpmWebLoginManager;

#[derive(OpenApi)]
#[openapi(
    paths(login_session, complete_login),
    components(schemas(NpmLoginSessionResponse, NpmLoginCompleteResponse))
)]
pub struct NpmAPI;

pub fn npm_routes(
    web_logins: Arc<NpmWebLoginManager>,
) -> Router<nr_repository::RepositoryRouterState> {
    Router::new()
        .route("/login/{session}", axum::routing::get(login_session))
        .route("/login/{session}", axum::routing::post(complete_login))
        .layer(Extension(web_logins))
}

/// What the login page needs to show the user before they approve.
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct NpmLoginSessionResponse {
    pub repository_id: Uuid,
    pub repository_name: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct NpmLoginCompleteResponse {
    pub repository_id: Uuid,
    pub repository_name: String,
}

/// Describes a pending npm login session.
#[utoipa::path(
    get,
    path = "/login/{session}",
    params(("session" = Uuid, Path, description = "The login session id npm was handed")),
    responses(
        (status = 200, description = "The pending session", body = NpmLoginSessionResponse),
        (status = 404, description = "Unknown or expired session"),
    )
)]
#[instrument(skip(site))]
pub async fn login_session(
    State(site): State<SiteContext>,
    Extension(web_logins): Extension<Arc<NpmWebLoginManager>>,
    Path(session): Path<Uuid>,
) -> Result<Response, InternalError> {
    let Some(repository_id) = web_logins.repository_for(session) else {
        return Ok(ResponseBuilder::not_found().empty());
    };
    // The repository is named rather than just identified so the page can tell the user what they
    // are about to grant a token for.
    let Some(repository) = DBRepository::get_by_id(repository_id, site.as_ref()).await? else {
        return Ok(ResponseBuilder::not_found().empty());
    };
    Ok(ResponseBuilder::ok().json(&NpmLoginSessionResponse {
        repository_id,
        repository_name: repository.name.to_string(),
    }))
}

/// Approves a pending npm login and releases a token to the waiting client.
#[utoipa::path(
    post,
    path = "/login/{session}",
    params(("session" = Uuid, Path, description = "The login session id npm was handed")),
    responses(
        (status = 200, description = "Login approved", body = NpmLoginCompleteResponse),
        (status = 401, description = "Not signed in"),
        (status = 403, description = "Cannot write to this repository"),
        (status = 404, description = "Unknown or expired session"),
    )
)]
#[instrument(skip(site))]
pub async fn complete_login(
    State(site): State<SiteContext>,
    Extension(web_logins): Extension<Arc<NpmWebLoginManager>>,
    auth: Option<Authentication>,
    Path(session): Path<Uuid>,
) -> Result<Response, InternalError> {
    let Some(auth) = auth else {
        return Ok(ResponseBuilder::unauthorized().empty());
    };
    let Some(repository_id) = web_logins.repository_for(session) else {
        return Ok(ResponseBuilder::not_found().empty());
    };
    // The token this hands out can publish, so approving a session requires the permission the
    // token would carry. Otherwise anyone with an account could complete a login started against
    // a repository they cannot write to.
    if !auth
        .has_action(RepositoryActions::Write, repository_id, site.as_ref())
        .await?
    {
        return Ok(MissingPermission::WriteRepository(repository_id).into_response());
    }
    let Some(repository) = DBRepository::get_by_id(repository_id, site.as_ref()).await? else {
        return Ok(ResponseBuilder::not_found().empty());
    };

    let (_, token) = NewRepositoryToken::new(
        auth.id,
        format!("npm login ({})", repository.name),
        repository_id,
        vec![RepositoryActions::Read, RepositoryActions::Write],
    )
    .insert(site.as_ref())
    .await?;

    if web_logins.complete(session, repository_id, token).is_err() {
        // Expired between the lookup above and here.
        return Ok(ResponseBuilder::not_found().empty());
    }
    info!(?repository_id, "Approved an npm web login");

    Ok(ResponseBuilder::ok().json(&NpmLoginCompleteResponse {
        repository_id,
        repository_name: repository.name.to_string(),
    }))
}

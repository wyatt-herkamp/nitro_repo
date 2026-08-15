//! Custom domains attached to a repository.
//!
//! A hostname registered here makes the repository answer on that host directly — see
//! [`crate::app::host_routing`] for what happens to a request once it matches.

use axum::{
    Json, Router,
    extract::{Path, State},
    response::{IntoResponse, Response},
    routing::get,
};
use nr_core::{
    database::entities::repository::{DBRepository, DBRepositoryHostname},
    repository::Hostname,
    user::{
        permissions::{HasPermissions, RepositoryActions},
        scopes::NRScope,
    },
};
use serde::Deserialize;
use tracing::{info, instrument};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{
    app::{
        NitroRepo,
        api::require_scope,
        authentication::Authentication,
        responses::{MissingPermission, RepositoryNotFound},
    },
    error::InternalError,
    utils::{ResponseBuilder, conflict::ConflictResponse, host::normalize_host},
};

pub fn hostname_routes() -> Router<NitroRepo> {
    Router::new()
        .route(
            "/{repository_id}/hostnames",
            get(list_hostnames).post(add_hostname),
        )
        .route(
            "/{repository_id}/hostnames/{hostname_id}",
            axum::routing::delete(delete_hostname),
        )
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct NewHostnameRequest {
    /// A bare hostname, without a scheme, port or path. Matched case-insensitively.
    pub hostname: String,
}

#[utoipa::path(
    get,
    path = "/{repository_id}/hostnames",
    params(("repository_id" = Uuid, Path, description = "The Repository ID")),
    responses(
        (status = 200, description = "The repository's custom domains", body = [DBRepositoryHostname]),
        (status = 404, description = "Repository not found"),
    )
)]
#[instrument]
pub async fn list_hostnames(
    State(site): State<NitroRepo>,
    auth: Authentication,
    Path(repository): Path<Uuid>,
) -> Result<Response, InternalError> {
    if !auth
        .has_action(RepositoryActions::Edit, repository, &site.database)
        .await?
    {
        return Ok(MissingPermission::EditRepository(repository).into_response());
    }
    if DBRepository::get_by_id(repository, site.as_ref())
        .await?
        .is_none()
    {
        return Ok(RepositoryNotFound::Uuid(repository).into_response());
    }
    let hostnames = DBRepositoryHostname::get_by_repository_id(&site.database, repository).await?;
    Ok(ResponseBuilder::ok().json(&hostnames))
}

#[utoipa::path(
    post,
    request_body = NewHostnameRequest,
    path = "/{repository_id}/hostnames",
    params(("repository_id" = Uuid, Path, description = "The Repository ID")),
    responses(
        (status = 201, description = "The registered domain", body = DBRepositoryHostname),
        (status = 400, description = "The hostname is not valid"),
        (status = 404, description = "Repository not found"),
        (status = 409, description = "That hostname is already in use"),
    )
)]
#[instrument]
pub async fn add_hostname(
    State(site): State<NitroRepo>,
    auth: Authentication,
    Path(repository): Path<Uuid>,
    Json(request): Json<NewHostnameRequest>,
) -> Result<Response, InternalError> {
    // A hostname is an instance-global resource, not a repository-scoped one: whoever can claim
    // one can decide where traffic for that name goes for everybody. So this asks for the same
    // permission creating and deleting a repository does, rather than the per-repository edit
    // permission that guards the read below.
    if !auth.is_admin_or_system_manager() {
        return Ok(MissingPermission::RepositoryManager.into_response());
    }
    if let Some(denied) = require_scope(&auth, NRScope::EditRepository, &site).await? {
        return Ok(denied);
    }
    if DBRepository::get_by_id(repository, site.as_ref())
        .await?
        .is_none()
    {
        return Ok(RepositoryNotFound::Uuid(repository).into_response());
    }

    let hostname = match Hostname::new(request.hostname) {
        Ok(hostname) => hostname,
        Err(error) => return Ok(ResponseBuilder::bad_request().body(error.to_string())),
    };

    // Claiming the instance's own hostname would route the web UI and every API call made through
    // that name into a repository, leaving the instance unmanageable — and unable to undo it.
    if is_instance_hostname(&site, &hostname) {
        return Ok(ConflictResponse::from("hostname").into_response());
    }

    // Checked before writing so this reports a conflict rather than a constraint violation; the
    // unique index is still what makes it correct under a race. The check is not scoped to
    // repository rows: the index is global, so a storage-scoped hostname is a real conflict too.
    if DBRepositoryHostname::is_hostname_taken(&site.database, &hostname).await? {
        return Ok(ConflictResponse::from("hostname").into_response());
    }

    let created = match DBRepositoryHostname::insert(&site.database, repository, &hostname).await {
        Ok(created) => created,
        Err(error) if is_unique_violation(&error) => {
            return Ok(ConflictResponse::from("hostname").into_response());
        }
        Err(error) => return Err(error.into()),
    };
    site.register_hostname(hostname.to_string(), repository);
    info!(%hostname, ?repository, "Registered a custom domain");

    Ok(ResponseBuilder::created().json(&created))
}

#[utoipa::path(
    delete,
    path = "/{repository_id}/hostnames/{hostname_id}",
    params(
        ("repository_id" = Uuid, Path, description = "The Repository ID"),
        ("hostname_id" = i32, Path, description = "The Hostname ID"),
    ),
    responses(
        (status = 204, description = "The domain was removed"),
        (status = 404, description = "No such domain on this repository"),
    )
)]
#[instrument]
pub async fn delete_hostname(
    State(site): State<NitroRepo>,
    auth: Authentication,
    Path((repository, hostname_id)): Path<(Uuid, i32)>,
) -> Result<Response, InternalError> {
    if !auth.is_admin_or_system_manager() {
        return Ok(MissingPermission::RepositoryManager.into_response());
    }
    if let Some(denied) = require_scope(&auth, NRScope::EditRepository, &site).await? {
        return Ok(denied);
    }
    // Scoped to the repository, so an id belonging to another repository is a 404 rather than a
    // deletion someone was not authorised to make through this route.
    let Some(deleted) =
        DBRepositoryHostname::delete(&site.database, hostname_id, repository).await?
    else {
        return Ok(ResponseBuilder::not_found().empty());
    };
    site.unregister_hostname(&deleted.hostname);
    info!(hostname = %deleted.hostname, ?repository, "Removed a custom domain");

    Ok(ResponseBuilder::no_content().empty())
}

/// Whether `hostname` is the host the instance itself is served on.
///
/// Skipped when `app_url` is unset — it defaults to an empty string, and an operator who has not
/// configured one has not told us which host to protect.
fn is_instance_hostname(site: &NitroRepo, hostname: &Hostname) -> bool {
    let app_url = { site.instance.lock().app_url.clone() };
    if app_url.is_empty() {
        return false;
    }
    let Ok(app_url) = url::Url::parse(&app_url) else {
        return false;
    };
    app_url
        .host_str()
        .and_then(normalize_host)
        .is_some_and(|instance_host| instance_host == hostname.as_str())
}

fn is_unique_violation(error: &sqlx::Error) -> bool {
    matches!(error, sqlx::Error::Database(error) if error.code().as_deref() == Some("23505"))
}

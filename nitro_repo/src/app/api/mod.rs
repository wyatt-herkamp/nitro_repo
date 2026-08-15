use axum::{
    Json,
    extract::{OriginalUri, Request, State},
    response::{IntoResponse, Response},
};
use http::{StatusCode, Uri};
use nr_core::{
    database::entities::user::NewUserRequest,
    user::scopes::{NRScope, ScopeDescription},
};
use serde::{Deserialize, Serialize, ser::SerializeStruct};
use strum::IntoEnumIterator;
use tower_http::cors::CorsLayer;
use tracing::{error, instrument};
use utoipa::ToSchema;
pub mod project;
pub mod repository;
pub mod search;
pub mod storage;
pub mod user;
pub mod user_management;
use super::{Instance, NitroRepo, NitroRepoState, authentication::password};
use crate::{
    error::InternalError,
    utils::{ResponseBuilder, api_error_response::APIErrorResponse},
};
/// Refuses a request whose token does not carry `scope`.
///
/// Returns `Some(response)` when the caller should be turned away, so a route reads as:
///
/// ```rust,ignore
/// if let Some(denied) = require_scope(&auth, NRScope::CreateRepository, &site).await? {
///     return Ok(denied);
/// }
/// ```
///
/// This is a *second* gate, not a replacement for the permission check. A scope bounds what a
/// token may do; whether the user may do it at all is still decided by their permissions, and both
/// have to pass.
pub async fn require_scope(
    auth: &super::authentication::Authentication,
    scope: NRScope,
    site: &NitroRepo,
) -> Result<Option<Response>, InternalError> {
    if auth.has_scope(scope, site.as_ref()).await? {
        return Ok(None);
    }
    Ok(Some(
        crate::app::responses::MissingPermission::Scope(scope).into_response(),
    ))
}

/// `site` rather than nothing, because the per-type routes come off the repository-type registry
/// and each arrives already carrying whatever state is private to that type.
pub fn api_routes(site: &NitroRepo) -> axum::Router<NitroRepo> {
    let mut router = axum::Router::new()
        .route("/info", axum::routing::get(info))
        .route("/info/scopes", axum::routing::get(scopes))
        .route("/install", axum::routing::post(install))
        .nest("/user", user::user_routes())
        .nest("/storage", storage::storage_routes())
        .nest(
            "/user-management",
            user_management::user_management_routes(),
        )
        .nest("/repository", repository::repository_routes())
        .nest("/project", project::project_routes())
        // The realm a Docker client is sent to by the `WWW-Authenticate` challenge. Under `/api`
        // rather than under `/v2` so it can never collide with an image named `token`.
        .nest("/docker", crate::repository::docker::api::docker_routes())
        .nest("/search", search::search_routes());

    // Each repository type may serve its own routes under `/api/{type}`. Mounted from the registry
    // rather than listed here so that a type's private state — npm's login-session manager — never
    // has to be reachable from the application state just to let its routes find it.
    for repository_type in site.inner.repository_types.iter() {
        if let Some(sub) = repository_type.api_router() {
            router = router.nest(
                &format!("/{}", repository_type.get_type()),
                sub.with_state(site.context()),
            );
        }
    }

    router
        .fallback(route_not_found)
        .layer(CorsLayer::very_permissive())
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
#[utoipa::path(
    get,
    path = "/api/info/scopes",
    responses(
        (status = 200, description = "List of all the scopes", body = [ScopeDescription])
    )
)]
pub async fn scopes() -> Response {
    let scopes = NRScope::iter()
        .map(|scope| scope.description())
        .collect::<Vec<_>>();
    let scopes = serde_json::to_string(&scopes).unwrap();
    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .body(scopes.into())
        .unwrap()
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
#[instrument(skip(site, request))]
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

#[derive(Debug)]
pub struct RouteNotFound {
    pub uri: Uri,
    pub method: http::Method,
}
impl Serialize for RouteNotFound {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut struct_ser = serializer.serialize_struct("RouteNotFound", 3)?;
        struct_ser.serialize_field("uri", &self.uri.to_string())?;
        struct_ser.serialize_field("path", &self.uri.path())?;
        struct_ser.serialize_field("method", &self.method.to_string())?;
        struct_ser.end()
    }
}
/// `/api/*` fall back is different than the rest of the site
/// The `/api` fallback.
///
/// A repository reached on one of its own hostnames owns that host's whole path space, and Cargo's
/// web API lives at `/api/v1/...` — paths this nest matches before host routing is ever consulted.
/// Without the fall-through, a Cargo registry on a custom domain served a `config.json` advertising
/// a `dl` and an `api` that answered `404` from inside the REST API, so `cargo publish` and every
/// download failed against URLs the registry itself had handed out.
///
/// Only *unmatched* `/api` paths get here, so the real API stays reachable on a custom domain — a
/// request for `/api/user/me` still hits `/api/user/me`.
async fn route_not_found(State(site): State<NitroRepo>, request: Request) -> Response {
    let host = crate::utils::host::request_host(
        request.headers(),
        request.uri(),
        site.general_security_settings.trust_forwarded_host,
    );
    if host
        .as_deref()
        .and_then(|host| site.repository_for_hostname(host))
        .is_some()
    {
        return host_route(site, request).await;
    }

    let response: APIErrorResponse<RouteNotFound, ()> = APIErrorResponse {
        message: "Not Found".into(),
        details: Some(RouteNotFound {
            uri: request.uri().clone(),
            method: request.method().clone(),
        }),
        ..Default::default()
    };
    ResponseBuilder::not_found()
        .error_reason("Route not found")
        .json(&response)
}

/// Hands a request back to host routing with its full path restored.
///
/// `nest` strips the prefix it matched, so a handler under `/api` sees `/v1/crates/new` rather than
/// `/api/v1/crates/new`. Host routing turns the path into a `StoragePath`, and the stripped form
/// would address the wrong thing entirely — `OriginalUri` is what the router recorded before the
/// rewrite.
async fn host_route(site: NitroRepo, mut request: Request) -> Response {
    if let Some(OriginalUri(original)) = request.extensions().get::<OriginalUri>().cloned() {
        *request.uri_mut() = original;
    }
    match crate::app::host_routing::host_or_frontend(State(site), request).await {
        Ok(response) => response,
        Err(error) => error.into_response(),
    }
}

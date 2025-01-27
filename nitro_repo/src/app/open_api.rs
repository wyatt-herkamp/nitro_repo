use crate::app::api::project::ProjectRoutes;
use crate::app::badge::BadgeRoutes;

use super::api;
use super::api::repository::RepositoryAPI;
use super::api::storage::StorageAPI;
use super::api::user::UserAPI;
use super::api::user_management::UserManagementAPI;
use axum::routing::get;
use axum::{
    response::{IntoResponse, Response},
    Json, Router,
};
use nr_core::database::entities::project::{
    versions::DBProjectVersion, DBProject, DBProjectMember,
};
use nr_core::database::entities::user::permissions::FullUserPermissions;
use nr_core::database::entities::user::NewUserRequest;
use nr_core::user::permissions::{RepositoryActions, UserPermissions};
use nr_core::user::scopes::{NRScope, ScopeDescription};
use utoipa::openapi::security::{ApiKey, ApiKeyValue, HttpAuthScheme, HttpBuilder, SecurityScheme};

use utoipa::{Modify, OpenApi};
#[derive(OpenApi)]
#[openapi(
    modifiers(&SecurityAddon),
    nest(
        (path = "/api/user", api = UserAPI, tags=["user"]),
        (path="/api/user-management", api = UserManagementAPI, tags=["user-management"]),
        (path = "/api/storage", api = StorageAPI, tags=["storage"]),
        (path = "/api/repository", api = RepositoryAPI, tags=["repository"]),
        (path="/badge", api = BadgeRoutes),
        (path="/api/project", api = ProjectRoutes, tags= ["project", "repository"]),
    ),
    paths(
        api::info,
        api::install,
        api::scopes,
    ),
    components(
        schemas(
            super::Instance,
            UserPermissions,
            api::InstallRequest,
            DBProject,
            DBProjectMember,
            DBProjectVersion,
            NewUserRequest, RepositoryActions, FullUserPermissions, ScopeDescription, NRScope
        )
    ),
    tags(
        (name="user",description= "Profile/User Access"),
        (name="user-management", description= "User Management. "),
        (name="storage", description= "Storage Management"),
        (name="repository",description= "Repository Management"),
        (name="config", description = "Repository Config Types"),
        (name="project", description = "Project Access"),
    )
)]
pub struct ApiDoc;

struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "api_key",
                SecurityScheme::Http(HttpBuilder::new().scheme(HttpAuthScheme::Bearer).build()),
            );
            components.add_security_scheme(
                "session",
                SecurityScheme::ApiKey(ApiKey::Cookie(ApiKeyValue::with_description("session",
                 "Session can also be applied via Authorization Header Formatted as `Authorization: session {session_id}`"))),
            );
        }
    }
}
#[cfg(feature = "utoipa-scalar")]
pub fn build_router<S>() -> axum::Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    use utoipa_scalar::{Scalar, Servable};

    Router::new()
        .route("/open-api-doc-raw", get(api_docs))
        .merge(Scalar::with_url("/scalar", ApiDoc::openapi()))
}
#[cfg(not(feature = "utoipa-scalar"))]
pub fn build_router<S>() -> axum::Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    Router::new().route("/open-api-doc-raw", get(api_docs))
}
async fn api_docs() -> Response {
    Json(ApiDoc::openapi()).into_response()
}

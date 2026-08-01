use axum::{
    Json, Router,
    response::{IntoResponse, Response},
    routing::get,
};
use nr_core::{
    database::entities::{
        project::{DBProject, members::DBProjectMember, versions::DBProjectVersion},
        user::{NewUserRequest, permissions::FullUserPermissions},
    },
    user::{
        permissions::{RepositoryActions, UserPermissions},
        scopes::{NRScope, ScopeDescription},
    },
};
use utoipa::{
    Modify, OpenApi,
    openapi::security::{ApiKey, ApiKeyValue, HttpAuthScheme, HttpBuilder, SecurityScheme},
};

use super::{
    api,
    api::{
        repository::RepositoryAPI, storage::StorageAPI, user::UserAPI,
        user_management::UserManagementAPI,
    },
};
use crate::app::{api::project::ProjectRoutes, badge::BadgeRoutes};
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
            NewUserRequest, RepositoryActions, FullUserPermissions, ScopeDescription, NRScope,
            // Both of these were referenced by generated paths but never registered, so the served
            // document contained `$ref`s pointing at components that did not exist. Any consumer
            // that resolves references — `openapi-typescript`, a client generator, Redocly —
            // refused the whole document over it.
            nr_core::storage::StoragePath,
            crate::utils::response::api_error_response::APIErrorResponseSchema,
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

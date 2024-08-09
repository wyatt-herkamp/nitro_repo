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
use nr_core::database::user::NewUserRequest;
use nr_core::user::permissions::{RepositoryActions, RepositoryPermission, UserPermissions};
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
    ),
    paths(
        api::info,
        api::install,
    ),
    components(
        schemas(
            super::Instance,
            RepositoryActions,
            RepositoryPermission,
            UserPermissions,
            api::InstallRequest,
            NewUserRequest
        )
    ),
    tags(
        (name="user",description= "Profile/User Access"),
        (name="user-management", description= "User Management. "),
        (name="storage", description= "Storage Management"),
        (name="repository",description= "Repository Management"),
        (name="config", description = "Repository Config Types")
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
pub fn build_router() -> axum::Router<crate::app::NitroRepo> {
    use utoipa_scalar::{Scalar, Servable};

    Router::new()
        .route("/open-api-doc-raw", get(api_docs))
        .merge(Scalar::with_url("/scalar", ApiDoc::openapi()))
}
#[cfg(not(feature = "utoipa-scalar"))]
pub fn build_router() -> axum::Router<crate::app::NitroRepo> {
    Router::new().route("/open-api-doc-raw", get(api_docs))
}
async fn api_docs() -> Response {
    Json(ApiDoc::openapi()).into_response()
}

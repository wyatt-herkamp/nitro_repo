use axum::{
    response::{IntoResponse, Response},
    routing::get,
};
use nr_core::user::permissions::HasPermissions;
use nr_storage::s3::regions::S3StorageRegion;
use strum::IntoEnumIterator;
use tracing::instrument;
use utoipa::OpenApi;

use crate::{
    app::{NitroRepo, authentication::Authentication, responses::MissingPermission},
    error::InternalError,
    utils::ResponseBuilder,
};

#[derive(OpenApi)]
#[openapi(paths(region_list), components(schemas(S3StorageRegion)))]
pub struct S3StorageAPI;
pub fn s3_storage_api() -> axum::Router<NitroRepo> {
    axum::Router::new().route("/regions", get(region_list))
}

#[utoipa::path(
    get,
    path = "/regions",
    responses(
        (status = 200, description = "A list of available regions for the S3 storage", body = Vec<S3StorageRegion>)
    )
)]
#[instrument]
pub async fn region_list(auth: Authentication) -> Result<Response, InternalError> {
    if !auth.is_admin_or_system_manager() {
        return Ok(MissingPermission::StorageManager.into_response());
    }
    let regions: Vec<_> = S3StorageRegion::iter().collect();
    Ok(ResponseBuilder::ok().json(&regions))
}

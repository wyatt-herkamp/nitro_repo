use axum::{
    body::Body,
    extract::{Path, State},
    response::{IntoResponse, Response},
    Json,
};
use http::StatusCode;
use tracing::instrument;

use crate::{
    app::{authentication::Authentication, NitroRepo},
    error::internal_error::InternalError,
};
pub fn config_routes() -> axum::Router<NitroRepo> {
    axum::Router::new()
        .route("/config/:key/schema", axum::routing::get(config_schema))
        .route(
            "/config/:key/validate",
            axum::routing::post(config_validate),
        )
        .route("/config/:key/default", axum::routing::get(config_default))
}
pub struct InvalidConfigType(String);
impl IntoResponse for InvalidConfigType {
    fn into_response(self) -> Response {
        Response::builder()
            .status(400)
            .body(Body::from(format!("Invalid Config Type: {}", self.0)))
            .unwrap()
    }
}

#[utoipa::path(
    get,
    path = "/config/{key}/schema",
    responses(
        (status = 200, description = "Returns a JSON Schema for the config type")
    ),
    params(
        ("key" = String, Path, description = "Config Key"),
    ),
)]
#[instrument]
pub async fn config_schema(
    State(site): State<NitroRepo>,
    Path(key): Path<String>,
) -> Result<Response, InternalError> {
    // TODO: Add Client side caching

    let Some(config_type) = site.get_repository_config_type(&key) else {
        return Ok(InvalidConfigType(key).into_response());
    };

    let schema = config_type
        .schema()
        .map(|schema| Json(schema).into_response())
        .unwrap_or_else(|| {
            Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body("No schema found".into())
                .unwrap()
        });
    Ok(schema)
}
#[utoipa::path(
    post,
    request_body = Value,
    path = "/config/{key}/validate",
    responses(
        (status = 200, description = "Returns a JSON Schema for the config type")
    ),
    params(
        ("key" = String, Path, description = "Config Key"),
    ),
)]
#[instrument]
pub async fn config_validate(
    State(site): State<NitroRepo>,
    Path(key): Path<String>,
    auth: Authentication,
    Json(config): Json<serde_json::Value>,
) -> Result<Response, InternalError> {
    //TODO: Check permissions
    let Some(config_type) = site.get_repository_config_type(&key) else {
        return Ok(InvalidConfigType(key).into_response());
    };

    let response = match config_type.validate_config(config) {
        Ok(_) => Response::builder()
            .status(StatusCode::NO_CONTENT)
            .body(Body::from("Valid Config"))
            .unwrap(),
        Err(err) => Response::builder()
            .status(400)
            .body(Body::from(err.to_string()))
            .unwrap(),
    };
    Ok(response)
}
#[utoipa::path(
    get,
    path = "/config/{key}/default",
    responses(
        (status = 200, description = "Returns the default config for the config type"),
    ),
    params(
        ("key" = String, Path, description = "Config Key"),
    ),
)]
#[instrument]
pub async fn config_default(
    State(site): State<NitroRepo>,
    Path(key): Path<String>,
) -> Result<Response, InternalError> {
    // TODO: Add Client side caching
    let Some(config_type) = site.get_repository_config_type(&key) else {
        return Ok(InvalidConfigType(key).into_response());
    };

    let default = match config_type.default() {
        Ok(ok) => Response::builder()
            .status(200)
            .header("Content-Type", "application/json")
            .body(Body::from(serde_json::to_string(&ok).unwrap()))
            .unwrap(),
        Err(err) => Response::builder()
            .status(500)
            .body(Body::from(err.to_string()))
            .unwrap(),
    };
    Ok(default)
}

use std::convert::Infallible;

use axum::{
    async_trait,
    extract::{FromRequest, FromRequestParts, Request, State},
};
use derive_more::From;
use http::{HeaderMap, StatusCode, Uri};
use http_body_util::combinators::BoxBody;
use nr_storage::{StorageFile, StoragePath};
use serde_json::Value;
use tracing::error;

use crate::{
    app::{authentication::RepositoryAuthentication, NitroRepo},
    error::internal_error::InternalError,
};
pub struct RepositoryRequest {
    pub request: Request,
    pub path: StoragePath,
    pub authentication: RepositoryAuthentication,
}
#[async_trait]
impl FromRequest<State<NitroRepo>> for RepositoryRequest {
    type Rejection = StatusCode;

    async fn from_request(
        req: Request,
        State(site): &State<NitroRepo>,
    ) -> Result<Self, Self::Rejection> {
        let (mut parts, body) = req.into_parts();
        let path = match StoragePath::try_from(parts.uri.clone()) {
            Ok(ok) => ok,
            Err(err) => {
                error!(?err, "Failed to parse path");
                return Err(StatusCode::BAD_REQUEST);
            }
        };

        let authentication =
            match RepositoryAuthentication::from_request_parts(&mut parts, site).await {
                Ok(ok) => ok,
                Err(err) => {
                    error!(?err, "Failed to authenticate request");
                    return Err(StatusCode::UNAUTHORIZED);
                }
            };

        Ok(Self {
            request: Request::from_parts(parts, body),
            path,
            authentication,
        })
    }
}

#[derive(Debug, From)]
pub enum RepoResponse {
    FileResponse(StorageFile),
    StringResponse(String, StatusCode),
    Json(Value, StatusCode),
    PUTResponse(bool, String),
}
impl RepoResponse {
    pub fn new_from_string(value: impl Into<String>, status: StatusCode) -> Self {
        Self::StringResponse(value.into(), status)
    }
}

use crate::{
    app::{
        authentication::{AuthenticationError, RepositoryAuthentication},
        NitroRepo,
    },
    error::internal_error::InternalError,
};
use axum::{
    async_trait,
    body::Body,
    extract::{FromRequest, FromRequestParts, Request, State},
    response::IntoResponse,
};
use bytes::Bytes;
use derive_more::From;
use http::{request::Parts, Response, StatusCode};
use http_body_util::BodyExt;
use nr_storage::{InvalidStoragePath, StorageFile, StoragePath};
use serde::Deserialize;
use serde_json::Value;
use tracing::error;
pub struct RepositoryRequest {
    pub parts: Parts,
    /// The body can be consumed only once
    pub body: Option<Body>,
    pub path: StoragePath,
    pub site: NitroRepo,
    pub authentication: RepositoryAuthentication,
}
impl RepositoryRequest {
    pub async fn body_as_bytes(&mut self) -> Result<Option<Bytes>, InternalError> {
        let Some(body) = self.body.take() else {
            return Ok(None);
        };
        let body = body
            .collect()
            .await
            .expect("I don't understand the error type. Fix Later")
            .to_bytes();
        Ok(Some(body))
    }
    pub async fn body_as_json<T: for<'a> Deserialize<'a>>(
        &mut self,
    ) -> Result<Option<T>, InternalError> {
        let body = self.body_as_bytes().await?;
        match body {
            None => Ok(None),
            Some(body) => Ok(serde_json::from_slice(&body)?),
        }
    }
}
impl AsRef<Parts> for RepositoryRequest {
    fn as_ref(&self) -> &Parts {
        &self.parts
    }
}
#[derive(Debug, From)]
pub enum RepositoryRequestError {
    InvalidPath(InvalidStoragePath),
    AuthorizationError(AuthenticationError),
}
impl IntoResponse for RepositoryRequestError {
    fn into_response(self) -> axum::response::Response {
        match self {
            Self::InvalidPath(err) => {
                error!(?err, "Failed to parse path");
                Response::builder()
                    .status(StatusCode::BAD_REQUEST)
                    .body(Body::from(err.to_string()))
                    .unwrap()
            }
            Self::AuthorizationError(err) => {
                error!(?err, "Failed to authenticate request");
                err.into_response()
            }
        }
    }
}
#[async_trait]
impl FromRequest<State<NitroRepo>> for RepositoryRequest {
    type Rejection = RepositoryRequestError;

    async fn from_request(
        req: Request,
        State(site): &State<NitroRepo>,
    ) -> Result<Self, Self::Rejection> {
        let (mut parts, body) = req.into_parts();
        let path = StoragePath::try_from(parts.uri.clone())?;
        let authentication = RepositoryAuthentication::from_request_parts(&mut parts, site).await?;
        Ok(Self {
            parts,
            body: Some(body),
            path,
            site: site.clone(),
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

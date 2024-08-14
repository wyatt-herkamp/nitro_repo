use std::{any::type_name, error::Error};

use crate::{
    app::{
        authentication::{AuthenticationError, RepositoryAuthentication},
        responses::RepositoryNotFound,
        NitroRepo, RepositoryStorageName,
    },
    error::{BadRequestErrors, IllegalStateError},
    repository::Repository,
    utils::headers::date_time::date_time_for_header,
};
use axum::{
    body::Body,
    debug_handler,
    extract::{Path, Request, State},
    response::{IntoResponse, Response},
};
use bytes::Bytes;
use derive_more::From;
use http::{
    header::{CONTENT_LENGTH, CONTENT_LOCATION, CONTENT_TYPE, ETAG, LAST_MODIFIED},
    request::Parts,
    HeaderValue, Method, StatusCode,
};
use http_body_util::BodyExt;
use nr_core::storage::{InvalidStoragePath, StoragePath};
use nr_storage::{StorageFile, StorageFileMeta, StorageFileReader};
use serde::Deserialize;
use serde_json::Value;
use tracing::{debug, error, info, instrument, trace};
use utoipa::openapi::info;

use super::RepositoryHandlerError;
#[derive(Debug, From)]
pub struct RepositoryRequestBody(Body);
impl RepositoryRequestBody {
    pub async fn body_as_bytes(self) -> Result<Bytes, RepositoryHandlerError> {
        // I am not sure if this error is user fault or server fault. I am going to assume it is a user fault for now
        let body = self.0.collect().await.map_err(BadRequestErrors::from)?;
        let bytes = body.to_bytes();
        Ok(bytes)
    }
    pub async fn body_as_json<T: for<'a> Deserialize<'a>>(
        self,
    ) -> Result<T, RepositoryHandlerError> {
        let body = self.body_as_bytes().await?;
        if body.is_empty() {
            let message = format!("Body is empty. Expected JSON for {}", type_name::<T>());
            return Err(BadRequestErrors::Other(message).into());
        }
        serde_json::from_slice(&body).map_err(RepositoryHandlerError::from)
    }
}
#[derive(Debug)]
pub struct RepositoryRequest {
    pub parts: Parts,
    /// The body can be consumed only once
    pub body: RepositoryRequestBody,
    pub path: StoragePath,
    pub authentication: RepositoryAuthentication,
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

fn response_file(meta: StorageFileMeta, content: StorageFileReader) -> Response<Body> {
    let last_modified = date_time_for_header(meta.modified());
    // TODO: Handle cache control headers
    let nr_storage::FileType::File {
        file_size,
        mime_type,
        file_hash,
    } = meta.file_type()
    else {
        return IllegalStateError("File has metadata of a directory").into_response();
    };
    let mut response = Response::builder()
        .status(StatusCode::OK)
        .header(CONTENT_LENGTH, file_size.to_string())
        .header(LAST_MODIFIED, last_modified);

    if let Some(etag) = &file_hash.sha256 {
        response = response.header(ETAG, etag);
    }
    if let Some(mime_type) = mime_type {
        response = response.header(CONTENT_TYPE, mime_type.to_string());
    }

    let Ok(file_size) = (*file_size).try_into() else {
        // So my guess. This software is running on a 32-bit system.
        // A. Why are you still on a 32-bit system?
        // B. How do you have a 4GB file hosted on a 32-bit system?
        // Either way. You are limited to the max usize for file sizes.
        // Now if this is a 64-bit system. Interesting. You have a file that is greater than 2^64 bytes.
        // Gigabit Internet won't help you now
        return IllegalStateError("File Size is greater than the systems max integer size")
            .into_response();
    };

    let body = Body::new(content.into_body(file_size));
    response.body(body).unwrap()
}

#[derive(Debug, From)]
pub enum RepoResponse {
    FileResponse(StorageFile),
    /// Should only be used for HEAD requests
    FileMetaResponse(StorageFileMeta),
    Json(Value, StatusCode),
    Generic(axum::response::Response),
    Unauthorized,
}
impl RepoResponse {
    /// Default Response Format
    pub fn into_response_default(self) -> Response {
        match self {
            Self::FileResponse(file) => match file {
                StorageFile::Directory { meta, files } => Response::builder()
                    .status(StatusCode::NOT_IMPLEMENTED)
                    .header(CONTENT_TYPE, mime::TEXT_HTML.to_string())
                    .body(Body::from("Build HTML Page listing"))
                    .unwrap(),
                StorageFile::File { meta, content } => response_file(meta, content),
            },
            Self::FileMetaResponse(meta) => {
                let last_modified = date_time_for_header(meta.modified());
                let mut response = Response::builder()
                    .status(StatusCode::OK)
                    .header(LAST_MODIFIED, last_modified);
                match meta.file_type() {
                    nr_storage::FileType::Directory { .. } => {
                        response.header(CONTENT_TYPE, mime::TEXT_HTML.to_string())
                    }
                    nr_storage::FileType::File {
                        file_hash,
                        file_size,
                        mime_type,
                    } => {
                        if let Some(etag) = &file_hash.sha256 {
                            response = response.header(ETAG, etag);
                        }
                        if let Some(mime_type) = mime_type {
                            response = response.header(CONTENT_TYPE, mime_type.to_string());
                        }
                        response.header(CONTENT_LENGTH, file_size.to_string())
                    }
                }
                .body(Body::empty())
                .unwrap()
            }
            Self::Json(json, status) => {
                let body = serde_json::to_string(&json).unwrap();
                Response::builder()
                    .status(status)
                    .header(CONTENT_LENGTH, body.len())
                    .header(CONTENT_TYPE, mime::APPLICATION_JSON.to_string())
                    .body(Body::from(body))
                    .unwrap()
            }
            Self::Generic(response) => response,
            Self::Unauthorized => Response::builder()
                .status(StatusCode::UNAUTHORIZED)
                .body(Body::from("Unauthorized"))
                .unwrap(),
        }
    }
    pub fn put_response(was_created: bool, location: impl AsRef<str>) -> Self {
        let status = if was_created {
            StatusCode::CREATED
        } else {
            StatusCode::NO_CONTENT
        };
        let header = match HeaderValue::from_str(location.as_ref()) {
            Ok(ok) => ok,
            Err(err) => {
                let location = location.as_ref();
                error!(?err, ?location, "Failed to create header for location");
                return Self::internal_error(err);
            }
        };

        Response::builder()
            .status(status)
            .header(CONTENT_LOCATION, header)
            .body(Body::empty())
            .unwrap()
            .into()
    }
    pub fn internal_error(error: impl Error) -> Self {
        error!(?error, "Internal Error");
        Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(Body::from(format!("Internal Error: {}", error)))
            .unwrap()
            .into()
    }
    pub fn basic_text_response(status: StatusCode, message: impl Into<String>) -> Self {
        Response::builder()
            .status(status)
            .body(Body::from(message.into()))
            .unwrap()
            .into()
    }
    pub fn indexing_not_allowed() -> Self {
        Self::basic_text_response(
            StatusCode::FORBIDDEN,
            "Indexing is not allowed for this repository",
        )
    }
    pub fn disabled_repository() -> Self {
        Self::basic_text_response(StatusCode::FORBIDDEN, "Repository is disabled")
    }
    pub fn unsupported_method_response(
        method: ::http::Method,
        repository_type: &str,
    ) -> RepoResponse {
        Response::builder()
            .status(StatusCode::METHOD_NOT_ALLOWED)
            .body(Body::from(format!(
                "Method {} is not supported for repository type {}",
                method, repository_type
            )))
            .unwrap()
            .into()
    }
}
impl From<Option<StorageFile>> for RepoResponse {
    fn from(file: Option<StorageFile>) -> Self {
        match file {
            Some(file) => RepoResponse::FileResponse(file),
            None => RepoResponse::basic_text_response(StatusCode::NOT_FOUND, "File not found"),
        }
    }
}
impl From<Option<StorageFileMeta>> for RepoResponse {
    fn from(meta: Option<StorageFileMeta>) -> Self {
        match meta {
            Some(meta) => RepoResponse::FileMetaResponse(meta),
            None => RepoResponse::basic_text_response(StatusCode::NOT_FOUND, "File not found"),
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
pub struct RepoRequestPath {
    storage: String,
    repository: String,
    path: Option<StoragePath>,
}
#[debug_handler]
#[instrument]
pub async fn handle_repo_request(
    State(site): State<NitroRepo>,
    Path(request_path): Path<RepoRequestPath>,
    authentication: RepositoryAuthentication,
    request: Request,
) -> Result<Response, RepositoryHandlerError> {
    debug!(?request_path, "Repository Request Happening");
    let RepoRequestPath {
        storage,
        repository,
        path,
    } = request_path;
    let Some(repository) = site
        .get_repository_from_names((storage.as_str(), repository.as_str()))
        .await?
    else {
        let not_found =
            RepositoryNotFound::from(RepositoryStorageName::from((storage, repository)));
        return Ok(not_found.into_response());
    };
    let method = request.method().clone();
    let (parts, body) = request.into_parts();
    let request = RepositoryRequest {
        parts,
        body: RepositoryRequestBody(body),
        path: path.unwrap_or_default(),
        authentication,
    };
    info!("Executing Request");
    let response = match method {
        Method::GET => repository.handle_get(request).await,
        Method::PUT => repository.handle_put(request).await,
        Method::DELETE => repository.handle_delete(request).await,
        Method::PATCH => repository.handle_patch(request).await,
        Method::HEAD => repository.handle_head(request).await,
        _ => repository.handle_other(request).await,
    };
    // TODO: If request is HTML, return HTML, If request is JSON, return JSON, else return text
    match response {
        Ok(response) => Ok(response.into_response_default()),
        Err(err) => {
            error!(?err, "Failed to handle request");
            Ok(err.into_response())
        }
    }
}
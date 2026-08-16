use std::error::Error;

use axum::{
    Router,
    body::Body,
    extract::{Path, Request, State},
    response::{IntoResponse, Response},
    routing::any,
};
use nr_web_core::{
    authentication::AuthenticationError,
    error::IllegalStateError,
    utils::{
        bad_request::BadRequestErrors, header::date_time::date_time_for_header,
        request_logging::request_span::RequestSpan,
    },
};

use crate::SiteContext;
pub mod directory_index;
pub mod repo_tracing;

use axum_extra::routing::RouterExt;
use bytes::Bytes;
use derive_more::From;
use http::{
    HeaderValue, StatusCode,
    header::{
        CONTENT_LENGTH, CONTENT_LOCATION, CONTENT_TYPE, ETAG, IF_MODIFIED_SINCE, IF_NONE_MATCH,
        LAST_MODIFIED, USER_AGENT,
    },
    request::Parts,
};
use http_body_util::BodyExt;
use nr_core::storage::{InvalidStoragePath, StoragePath};
use nr_storage::{FileFileType, FileType, StorageFile, StorageFileMeta, StorageFileReader};
use serde::Deserialize;
use tracing::{Level, Span, debug, debug_span, error, event, instrument};
mod repo_auth;
pub use repo_auth::*;

use super::{
    DynRepository, RepositoryHandlerError, RepositoryNotFound, RepositoryRouterState,
    RepositoryStorageName, repo_tracing::RepositoryRequestTracing,
};
/// `RepositoryRouterState` rather than the application state: everything below needs a database,
/// the site context, and the ability to turn a name into a repository — and nothing more.
pub fn repository_router() -> axum::Router<RepositoryRouterState> {
    Router::new()
        .route("/{storage}/{repository}/{*path}", any(handle_repo_request))
        .route_with_tsr("/{storage}/{repository}", any(handle_repo_request))
}

#[derive(Debug, From)]
pub struct RepositoryRequestBody(Body);
impl RepositoryRequestBody {
    #[instrument]
    pub async fn body_as_bytes(self) -> Result<Bytes, RepositoryHandlerError> {
        // I am not sure if this error is user fault or server fault. I am going to assume it is a user fault for now
        let body = self.0.collect().await.map_err(BadRequestErrors::from)?;
        let bytes = body.to_bytes();
        Ok(bytes)
    }
    #[cfg(not(debug_assertions))]
    #[instrument]
    pub async fn body_as_json<T: for<'a> Deserialize<'a>>(
        self,
    ) -> Result<T, RepositoryHandlerError> {
        let body = self.body_as_bytes().await?;
        serde_json::from_slice(&body).map_err(RepositoryHandlerError::from)
    }
    /// In Debug mode we convert to a string so we can debug it
    #[cfg(debug_assertions)]
    #[instrument]
    pub async fn body_as_json<T: for<'a> Deserialize<'a>>(
        self,
    ) -> Result<T, RepositoryHandlerError> {
        let body = self.body_as_string().await?;
        debug!(?body, "Body as JSON");
        Ok(serde_json::from_str(&body).map_err(BadRequestErrors::from)?)
    }
    #[instrument]
    pub async fn body_as_string(self) -> Result<String, RepositoryHandlerError> {
        let body = self.body_as_bytes().await?;
        let body = String::from_utf8(body.to_vec()).map_err(BadRequestErrors::from)?;
        Ok(body)
    }
}

#[derive(Debug)]
pub struct RepositoryRequest {
    pub parts: Parts,
    /// The body can be consumed only once
    pub body: RepositoryRequestBody,
    pub path: StoragePath,
    pub authentication: RepositoryAuthentication,
    pub trace: RepositoryRequestTracing,
}
impl RepositoryRequest {
    #[inline(always)]
    pub fn headers(&self) -> &http::HeaderMap {
        &self.parts.headers
    }
    pub fn user_agent_as_string(&self) -> Result<Option<&str>, BadRequestErrors> {
        let Some(header_value) = self.parts.headers.get(USER_AGENT) else {
            return Ok(None);
        };
        header_value
            .to_str()
            .map(Some)
            .map_err(BadRequestErrors::from)
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
    BadRequestErrors(BadRequestErrors),
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
            Self::BadRequestErrors(err) => {
                error!(?err, "Bad Request Error");
                err.into_response()
            }
        }
    }
}

/// Redirects a directory request that arrived without a trailing slash.
///
/// Returns `None` when the path already ends in `/`, or is the repository root (which the router
/// reaches by a path that is empty rather than one that needs a slash appended).
///
/// The `Location` is relative — `kingtux/` from `/repositories/s/r/dev/kingtux` resolves to
/// `/repositories/s/r/dev/kingtux/` — so this does not need to know the URL prefix it was mounted
/// under, which the response layer has no access to.
fn redirect_directory_to_slash(path: &StoragePath) -> Option<Response<Body>> {
    if path.is_directory() || path.number_of_components() == 0 {
        return None;
    }
    let last = path.clone().into_iter().next_back()?;
    let location = format!("{}/", last.as_ref());
    let location = HeaderValue::from_str(&location).ok()?;
    Some(
        Response::builder()
            .status(StatusCode::MOVED_PERMANENTLY)
            .header(http::header::LOCATION, location)
            .body(Body::empty())
            .unwrap(),
    )
}

fn response_file(
    meta: StorageFileMeta<FileFileType>,
    content: StorageFileReader,
    context: &ResponseContext,
) -> Response<Body> {
    let last_modified = date_time_for_header(meta.modified());
    let FileFileType {
        file_size,
        mime_type,
        file_hash,
    } = meta.file_type();

    // An artifact at a given coordinate is normally immutable, so a client that already has it
    // should not pull it again. Nothing answered conditional requests before, which meant every
    // `mvn` run re-downloaded everything it had already cached.
    if context.is_fresh(
        file_hash.sha2_256.as_deref(),
        last_modified.to_str().unwrap_or_default(),
    ) {
        let mut response = Response::builder()
            .status(StatusCode::NOT_MODIFIED)
            .header(LAST_MODIFIED, &last_modified);
        if let Some(etag) = &file_hash.sha2_256 {
            response = response.header(ETAG, etag);
        }
        return response.body(Body::empty()).unwrap();
    }

    let mut response = Response::builder()
        .status(StatusCode::OK)
        .header(CONTENT_LENGTH, file_size.to_string())
        .header(LAST_MODIFIED, last_modified);

    if let Some(etag) = &file_hash.sha2_256 {
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

/// What the response needs to know about the request that produced it.
///
/// [RepoResponse] is built by a repository handler that has already consumed the request, but
/// rendering it correctly needs two things from the original: the path (so a directory listing can
/// link relative to it) and the cache validators (so an unchanged artifact can answer `304`).
#[derive(Debug, Clone, Default)]
pub struct ResponseContext {
    pub path: StoragePath,
    if_none_match: Option<String>,
    if_modified_since: Option<String>,
}

impl ResponseContext {
    pub fn new(path: StoragePath, parts: &Parts) -> Self {
        let header = |name: &http::HeaderName| {
            parts
                .headers
                .get(name)
                .and_then(|value| value.to_str().ok())
                .map(str::to_owned)
        };
        Self {
            path,
            if_none_match: header(&IF_NONE_MATCH),
            if_modified_since: header(&IF_MODIFIED_SINCE),
        }
    }

    /// Whether the client already holds this exact version.
    ///
    /// `If-None-Match` wins over `If-Modified-Since` when both are sent, per RFC 9110: an ETag is
    /// an exact identity, a timestamp only has one-second resolution and cannot see a file that
    /// was rewritten within the same second.
    fn is_fresh(&self, etag: Option<&str>, last_modified: &str) -> bool {
        if let Some(if_none_match) = &self.if_none_match {
            let Some(etag) = etag else {
                return false;
            };
            return if_none_match == "*"
                || if_none_match
                    .split(',')
                    .map(|candidate| candidate.trim().trim_start_matches("W/").trim_matches('"'))
                    .any(|candidate| candidate == etag.trim_matches('"'));
        }
        self.if_modified_since
            .as_deref()
            .is_some_and(|since| since == last_modified)
    }
}

#[derive(Debug, From)]
pub enum RepoResponse {
    FileResponse(Box<StorageFile>),
    FileMetaResponse(Box<StorageFileMeta<FileType>>),
    Other(axum::response::Response),
}
impl From<StorageFileMeta<FileType>> for RepoResponse {
    fn from(meta: StorageFileMeta<FileType>) -> Self {
        RepoResponse::FileMetaResponse(Box::new(meta))
    }
}
impl RepoResponse {
    /// Response format used when a handler does not build one itself.
    pub fn into_response_default(self) -> Response {
        self.into_response_with(&ResponseContext::default())
    }

    /// Renders the response, using the request context for cache validation and directory links.
    pub fn into_response_with(self, context: &ResponseContext) -> Response {
        match self {
            Self::FileResponse(file) => match *file {
                StorageFile::Directory { meta, files } => {
                    // Entry links are relative, so a URL that does not end in `/` would resolve
                    // them against the *parent* directory. Redirect to the canonical form first,
                    // which is what nginx and Apache do for exactly this reason.
                    if let Some(redirect) = redirect_directory_to_slash(&context.path) {
                        return redirect;
                    }
                    // Used to be `501 Not Implemented` with the body "Build HTML Page listing".
                    // Maven walks directory listings to resolve version ranges and LATEST/RELEASE.
                    let body = directory_index::render(&context.path, &meta, &files);
                    Response::builder()
                        .status(StatusCode::OK)
                        .header(CONTENT_TYPE, mime::TEXT_HTML_UTF_8.to_string())
                        .header(LAST_MODIFIED, date_time_for_header(meta.modified()))
                        .body(Body::from(body))
                        .unwrap()
                }
                StorageFile::File { meta, content } => response_file(meta, content, context),
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
                    nr_storage::FileType::File(FileFileType {
                        file_hash,
                        file_size,
                        mime_type,
                    }) => {
                        if let Some(etag) = &file_hash.sha2_256 {
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
            Self::Other(response) => response,
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
    pub fn www_authenticate(value: &str) -> Self {
        Response::builder()
            .status(StatusCode::UNAUTHORIZED)
            .header("WWW-Authenticate", value)
            .body(Body::from("Unauthorized"))
            .unwrap()
            .into()
    }
    pub fn unauthorized() -> Self {
        Response::builder()
            .status(StatusCode::UNAUTHORIZED)
            .body(Body::from("Unauthorized"))
            .unwrap()
            .into()
    }
    pub fn forbidden() -> Self {
        Response::builder()
            .status(StatusCode::FORBIDDEN)
            .body(Body::from(
                "You do not have permission to access this repository",
            ))
            .unwrap()
            .into()
    }
    pub fn require_auth_token() -> Self {
        Response::builder()
            .status(StatusCode::UNAUTHORIZED)
            .body(Body::from(
                "Authentication Token is required for this repository.",
            ))
            .unwrap()
            .into()
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
impl From<Result<Response, http::Error>> for RepoResponse {
    fn from(result: Result<Response, http::Error>) -> Self {
        match result {
            Ok(response) => RepoResponse::Other(response),
            Err(err) => {
                error!(?err, "Failed to create response");
                RepoResponse::internal_error(err)
            }
        }
    }
}
impl From<StorageFile> for RepoResponse {
    fn from(file: StorageFile) -> Self {
        RepoResponse::FileResponse(Box::new(file))
    }
}
impl From<Option<StorageFile>> for RepoResponse {
    fn from(file: Option<StorageFile>) -> Self {
        match file {
            Some(file) => RepoResponse::FileResponse(Box::new(file)),
            None => RepoResponse::basic_text_response(StatusCode::NOT_FOUND, "File not found"),
        }
    }
}

impl From<Option<StorageFileMeta<FileType>>> for RepoResponse {
    fn from(meta: Option<StorageFileMeta<FileType>>) -> Self {
        match meta {
            Some(meta) => RepoResponse::FileMetaResponse(Box::new(meta)),
            None => RepoResponse::basic_text_response(StatusCode::NOT_FOUND, "File not found"),
        }
    }
}
#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
pub struct RepoRequestPath {
    storage: String,
    repository: String,
    #[serde(default)]
    path: Option<StoragePath>,
}

pub async fn handle_repo_request(
    State(state): State<RepositoryRouterState>,
    Path(request_path): Path<RepoRequestPath>,
    parent_span: Option<RequestSpan>,
    authentication: RepositoryAuthentication,
    request: Request,
) -> Result<Response, RepositoryHandlerError> {
    let parent_span = parent_span.map(|span| span.0).unwrap_or(Span::current());
    let request_debug = debug_span!(
        target: "nitro_repo::repository::requests",
        parent: &parent_span,
        "Repository Request",
        request_path = ?request_path,
        authentication = ?authentication
    );
    let entered_guard = request_debug.enter();
    debug!(?request_path, "Repository Request Happening");
    let RepoRequestPath {
        storage,
        repository,
        path,
    } = request_path;
    let names = RepositoryStorageName::from((storage, repository));
    let Some(repository) = state.resolver.repository_from_names(&names).await? else {
        let not_found = RepositoryNotFound::from(names);
        return Ok(not_found.into_response());
    };
    drop(entered_guard);
    Ok(dispatch_repository_request(
        &state.context,
        repository,
        path.unwrap_or_default(),
        authentication,
        &parent_span,
        request,
    )
    .await)
}

/// Runs a resolved repository against a request.
///
/// Shared by the two ways a request reaches a repository: the `/repositories/{storage}/{repository}`
/// path, and a request whose `Host` is a hostname registered to the repository. The two differ only
/// in how they arrive at the repository and at the path — everything from here on, including the
/// active check, tracing and the method dispatch, is identical.
///
/// Nothing below is aware of the URL prefix the request arrived under: [`ResponseContext`] only
/// reads conditional-request headers, and `redirect_directory_to_slash` emits a relative
/// `Location`. That is what lets a host-routed request reuse this unchanged.
pub async fn dispatch_repository_request(
    site: &SiteContext,
    repository: DynRepository,
    path: StoragePath,
    authentication: RepositoryAuthentication,
    parent_span: &Span,
    request: Request,
) -> Response {
    if !repository.is_active() {
        return RepoResponse::disabled_repository().into_response_default();
    }
    let (parts, body) = request.into_parts();
    let response_context = ResponseContext::new(path.clone(), &parts);
    let trace =
        RepositoryRequestTracing::new(&repository, parent_span, site.repository_metrics.clone());
    trace.path(&path);
    let request = RepositoryRequest {
        parts,
        body: RepositoryRequestBody(body),
        path,
        authentication,
        trace: trace.clone(),
    };
    let response = {
        let _guard = trace.span.enter();
        // The method dispatch that used to be spelled out here now lives behind
        // `RepositoryHandler::handle`, which does the same match on `request.parts.method`.
        let response = repository.handle(request).await;
        match &response {
            Ok(_) => {
                trace.ok();
            }
            Err(err) => {
                trace.error(err);
            }
        }
        event!(Level::DEBUG, "Repository Request Completed");
        response
    };
    let _guard = parent_span.enter();
    match response {
        Ok(response) => response.into_response_with(&response_context),
        Err(err) => {
            error!(?err, "Failed to handle request");
            err.into_response()
        }
    }
}

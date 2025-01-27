use std::{
    future::Future,
    io::Write,
    net::SocketAddr,
    task::Poll,
    time::Duration,
    vec::{self, IntoIter},
};
mod ws;
use axum::{
    extract::{ConnectInfo, Path, Query, State, WebSocketUpgrade},
    response::{IntoResponse, Response},
    routing::{any, get},
};
use axum_extra::{headers::UserAgent, TypedHeader};
use bytes::{BufMut, Bytes, BytesMut};
use futures::Stream;
use http_body::Frame;
use http_body_util::StreamBody;
use nr_core::{
    repository::{
        browse::{BrowseFile, BrowseResponse},
        project::ProjectResolution,
    },
    storage::StoragePath,
};
use nr_storage::{DirectoryListStream, DynDirectoryListStream, Storage, StorageFile};
use pin_project::pin_project;
use serde::{Deserialize, Serialize};
use tokio::time::{sleep, Instant, Sleep};
use tracing::{debug, event, info, info_span, instrument, Level};
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

use crate::{
    app::{
        api::info,
        authentication::Authentication,
        logging::request_logging::RequestSpan,
        responses::{MissingPermission, RepositoryNotFound},
        NitroRepo,
    },
    error::InternalError,
    repository::{utils::can_read_repository, Repository},
    utils::response_builder::ResponseBuilder,
};
pub fn browse_routes() -> axum::Router<NitroRepo> {
    axum::Router::new()
        .route("/browse-ws/{repository_id}", any(ws_handler))
        .route("/browse/{repository_id}", get(browse))
        .route("/browse/{repository_id}/", get(browse))
        .route("/browse/{repository_id}/{*path}", get(browse))
        .route("/browse-stream/{repository_id}", get(browse_stream))
        .route("/browse-stream/{repository_id}/", get(browse_stream))
        .route("/browse-stream/{repository_id}/{*path}", get(browse_stream))
}
#[derive(Debug, Deserialize, Clone, ToSchema, IntoParams)]
#[serde(default)]
#[into_params(style = Form, parameter_in = Query)]
pub struct BrowseParams {
    #[schema(default = true)]
    #[param(default = true)]
    pub check_for_project: bool,
}
impl Default for BrowseParams {
    fn default() -> Self {
        Self {
            check_for_project: true,
        }
    }
}
#[derive(Debug, Serialize, Deserialize, Clone, IntoParams)]
#[into_params(parameter_in = Path)]
pub struct BrowsePath {
    /// The repository to browse
    pub repository_id: Uuid,
    /// The path to browse
    pub path: Option<StoragePath>,
}
/// Browses a repository at a specified path
#[utoipa::path(
    get,
    path = "/browse/{repository_id}/{path}",
    params(
        BrowseParams,
        BrowsePath
    ),
    responses(
        (status = 200, description = "File listing", body = BrowseResponse),
        (status = 404, description = "Repository not found or file not found"),
        (status = 403, description = "Missing permission"),
    ),
)]
#[instrument]
async fn browse(
    State(site): State<NitroRepo>,
    auth: Option<Authentication>,
    Path(browse_path): Path<BrowsePath>,
    Query(params): Query<BrowseParams>,
) -> Result<Response, InternalError> {
    let Some(repository) = site.get_repository(browse_path.repository_id) else {
        return Ok(RepositoryNotFound::Uuid(browse_path.repository_id).into_response());
    };
    if !can_read_repository(
        auth,
        repository.visibility(),
        repository.id(),
        site.as_ref(),
    )
    .await?
    {
        return Ok(MissingPermission::ReadRepository(repository.id()).into_response());
    }
    let repository_storage = repository.get_storage();
    let path = browse_path.path.unwrap_or_default();
    let Some(file) = repository_storage.open_file(repository.id(), &path).await? else {
        return Ok(ResponseBuilder::not_found().empty());
    };
    let files = match file {
        StorageFile::Directory { files, .. } => files.into_iter().map(BrowseFile::from).collect(),
        StorageFile::File { meta, .. } => {
            vec![BrowseFile::from(meta)]
        }
    };
    let project_resolution = if params.check_for_project {
        event!(Level::DEBUG, "Checking for project and version");
        match repository.resolve_project_and_version_for_path(&path).await {
            Ok(ok) => Some(ok),
            Err(err) => {
                event!(
                    Level::ERROR,
                    ?err,
                    ?path,
                    "Failed to resolve project and version for path"
                );
                Some(ProjectResolution::default())
            }
        }
    } else {
        None
    };

    let body = BrowseResponse {
        files,
        project_resolution,
    };
    Ok(ResponseBuilder::ok().json(&body))
}
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BrowseStreamPrimaryData {
    pub project_resolution: Option<ProjectResolution>,
    pub number_of_files: usize,
}
#[instrument]
async fn browse_stream(
    State(site): State<NitroRepo>,
    auth: Option<Authentication>,
    Path(browse_path): Path<BrowsePath>,
    Query(params): Query<BrowseParams>,
) -> Result<Response, InternalError> {
    let Some(repository) = site.get_repository(browse_path.repository_id) else {
        return Ok(RepositoryNotFound::Uuid(browse_path.repository_id).into_response());
    };
    if !can_read_repository(
        auth,
        repository.visibility(),
        repository.id(),
        site.as_ref(),
    )
    .await?
    {
        return Ok(MissingPermission::ReadRepository(repository.id()).into_response());
    }
    let repository_storage = repository.get_storage();
    let path = browse_path.path.unwrap_or_default();

    let project_resolution = if params.check_for_project {
        event!(Level::DEBUG, "Checking for project and version");
        match repository.resolve_project_and_version_for_path(&path).await {
            Ok(ok) => Some(ok),
            Err(err) => {
                event!(
                    Level::ERROR,
                    ?err,
                    ?path,
                    "Failed to resolve project and version for path"
                );
                Some(ProjectResolution::default())
            }
        }
    } else {
        None
    };
    let mut response = Response::builder().status(http::StatusCode::OK);

    let Some(files) = repository_storage
        .stream_directory(repository.id(), &path)
        .await?
    else {
        return Ok(ResponseBuilder::not_found().empty());
    };
    let primary_response = BrowseStreamPrimaryData {
        project_resolution,
        number_of_files: files.number_of_files() as usize,
    };

    response = response.header("Content-Type", "application/jsonstream");
    let stream = FileStream {
        files: files,
        span: tracing::Span::current(),
        primary_response: Some(primary_response),
    };
    Ok(response
        .body(axum::body::Body::new(StreamBody::new(stream)))
        .unwrap())
}
#[pin_project]
pub struct FileStream {
    #[pin]
    pub files: DynDirectoryListStream,
    span: tracing::Span,
    primary_response: Option<BrowseStreamPrimaryData>,
}
impl Stream for FileStream {
    type Item = Result<Frame<Bytes>, InternalError>;

    fn poll_next(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        let this = self.project();
        let _guard = this.span.enter();
        if let Some(primary_response) = this.primary_response.take() {
            let mut bytes = BytesMut::new().writer();
            serde_json::to_writer(&mut bytes, &primary_response).unwrap();
            bytes.write_all(b"\n").unwrap();
            let bytes = bytes.into_inner().freeze();
            return std::task::Poll::Ready(Some(Ok(Frame::data(bytes))));
        }

        let file = match this.files.poll_next(cx) {
            std::task::Poll::Ready(Some(file)) => file,
            std::task::Poll::Ready(None) => return std::task::Poll::Ready(None),
            std::task::Poll::Pending => return std::task::Poll::Pending,
        };

        match file {
            Ok(Some(file)) => {
                debug!(?file, "Sending file");
                let file = BrowseFile::from(file);
                let mut bytes = BytesMut::new().writer();
                serde_json::to_writer(&mut bytes, &file).unwrap();
                bytes.write_all(b"\n").unwrap();
                let bytes = bytes.into_inner().freeze();
                std::task::Poll::Ready(Some(Ok(Frame::data(bytes))))
            }
            Ok(None) => Poll::Ready(Some(Ok(Frame::data(Bytes::new())))),
            Err(err) => {
                event!(Level::ERROR, ?err, "Failed to read file");
                std::task::Poll::Ready(Some(Err(err.into())))
            }
        }
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        (
            self.files.number_of_files() as usize,
            Some(self.files.number_of_files() as usize),
        )
    }
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    user_agent: Option<TypedHeader<UserAgent>>,
    Path(repository_id): Path<Uuid>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(site): State<NitroRepo>,
    RequestSpan(span): RequestSpan,
) -> Result<Response, InternalError> {
    let ws_span = info_span!(
        parent: span,
        "Browse WS",
    );
    let _guard = ws_span.enter();
    let user_agent = if let Some(TypedHeader(user_agent)) = user_agent {
        user_agent.to_string()
    } else {
        String::from("Unknown browser")
    };
    info!(?addr, ?user_agent, "Websocket connection");
    let Some(repository) = site.get_repository(repository_id) else {
        return Ok(RepositoryNotFound::Uuid(repository_id).into_response());
    };
    drop(_guard);
    // finalize the upgrade process by returning upgrade callback.
    // we can customize the callback by sending additional info such as address.
    Ok(ws.on_upgrade(move |socket| ws::handle_socket(socket, addr, repository, ws_span)))
}

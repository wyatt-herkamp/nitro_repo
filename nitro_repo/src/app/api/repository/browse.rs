use std::net::SocketAddr;
mod ws;
use axum::{
    extract::{ConnectInfo, Path, Query, State, WebSocketUpgrade},
    response::{IntoResponse, Response},
    routing::{any, get},
};
use axum_extra::{TypedHeader, headers::UserAgent};

use nr_core::{
    repository::{
        browse::{BrowseFile, BrowseResponse},
        project::ProjectResolution,
    },
    storage::StoragePath,
};
use nr_storage::{Storage, StorageFile};
use serde::{Deserialize, Serialize};
use tracing::{Level, Span, event, info, info_span, instrument};
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

use crate::{
    app::{
        NitroRepo,
        authentication::Authentication,
        logging::request_logging::RequestId,
        responses::{MissingPermission, RepositoryNotFound},
    },
    error::InternalError,
    repository::{Repository, utils::can_read_repository},
    utils::response_builder::ResponseBuilder,
};
pub fn browse_routes() -> axum::Router<NitroRepo> {
    axum::Router::new()
        .route("/browse-ws/{repository_id}", any(browse_ws_handler))
        .route("/browse/{repository_id}", get(browse))
        .route("/browse/{repository_id}/", get(browse))
        .route("/browse/{repository_id}/{*path}", get(browse))
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
        &auth,
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
async fn browse_ws_handler(
    ws: WebSocketUpgrade,
    user_agent: Option<TypedHeader<UserAgent>>,
    Path(repository_id): Path<Uuid>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(site): State<NitroRepo>,
    RequestId(request_id): RequestId,
) -> Result<Response, InternalError> {
    let user_agent = if let Some(TypedHeader(user_agent)) = user_agent {
        user_agent.to_string()
    } else {
        String::from("Unknown browser")
    };
    info!(?addr, ?user_agent, "Websocket connection");
    let Some(repository) = site.get_repository(repository_id) else {
        return Ok(RepositoryNotFound::Uuid(repository_id).into_response());
    };
    // finalize the upgrade process by returning upgrade callback.
    // we can customize the callback by sending additional info such as address.
    let ws_span = info_span!(
        parent: Span::none(),
        "Browse WS",
        request_id = %request_id,
        http.client_id = %addr,
        repository_id = %repository_id,
        http.user_agent = %user_agent,
    );
    Ok(ws.on_upgrade(move |socket| ws::handle_socket(socket, addr, repository, site, ws_span)))
}

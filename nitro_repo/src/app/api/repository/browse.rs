use axum::{
    body::Body,
    extract::{Path, Query, State},
    response::{IntoResponse, Response},
    routing::get,
};
use chrono::{DateTime, FixedOffset};
use http::StatusCode;
use nr_core::{
    database::project::{DBProject, DBProjectVersion},
    storage::StoragePath,
};
use nr_storage::{FileHashes, FileType, SerdeMime, Storage, StorageFile, StorageFileMeta};
use serde::{Deserialize, Serialize};
use tracing::{debug, error, instrument};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{
    app::{
        authentication::Authentication,
        responses::{MissingPermission, RepositoryNotFound, ResponseBuilderExt},
        NitroRepo,
    },
    error::InternalError,
    repository::{utils::can_read_repository, Repository},
};
pub fn browse_routes() -> axum::Router<NitroRepo> {
    axum::Router::new()
        .route("/browse/:repository_id", get(browse))
        .route("/browse/:repository_id/", get(browse))
        .route("/browse/:repository_id/*path", get(browse))
}
#[derive(Debug, Deserialize, Clone, ToSchema)]
#[serde(default)]
pub struct BrowseParams {
    pub check_for_project: bool,
}
impl Default for BrowseParams {
    fn default() -> Self {
        Self {
            check_for_project: true,
        }
    }
}
#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct BrowsePath {
    pub repository_id: Uuid,
    pub path: Option<StoragePath>,
}
#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
#[serde(tag = "type", content = "value")]
pub enum BrowseFile {
    File {
        name: String,
        file_size: u64,
        mime_type: Option<SerdeMime>,
        file_hash: FileHashes,
        /// Last time it was modified.
        modified: DateTime<FixedOffset>,
        /// The first time it was created.
        created: DateTime<FixedOffset>,
    },
    Directory {
        name: String,
        number_of_files: usize,
        //modified: DateTime<FixedOffset>,
    },
}
impl From<StorageFileMeta> for BrowseFile {
    fn from(meta: StorageFileMeta) -> Self {
        let StorageFileMeta {
            name,
            file_type,
            modified,
            created,
            ..
        } = meta;
        match file_type {
            FileType::File {
                file_size,
                mime_type,
                file_hash,
            } => BrowseFile::File {
                name: name,
                file_size: file_size,
                mime_type: mime_type,
                file_hash: file_hash,
                modified: modified,
                created: created,
            },
            FileType::Directory { file_count } => BrowseFile::Directory {
                name: name,
                number_of_files: file_count as usize,
            },
        }
    }
}
#[derive(Debug, Serialize, Clone, ToSchema)]
pub struct BrowseResponse {
    pub files: Vec<BrowseFile>,
    /// The project contained in the path
    pub project: Option<DBProject>,
    /// The version of the project contained in the path
    pub version: Option<DBProjectVersion>,
}
#[utoipa::path(
    get,
    path = "/browse/{repository_id}/{path}",
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
        let response = Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::empty())
            .unwrap();
        return Ok(response);
    };
    let files = match file {
        StorageFile::Directory { files, .. } => files.into_iter().map(BrowseFile::from).collect(),
        StorageFile::File { meta, .. } => {
            vec![BrowseFile::from(meta)]
        }
    };
    let (project, version) = if params.check_for_project {
        match repository.resolve_project_and_version_for_path(path).await {
            Ok(ok) => (ok.project, ok.version),
            Err(err) => {
                error!("Error resolving project and version: {:?}", err);
                (None, None)
            }
        }
    } else {
        (None, None)
    };
    let body = BrowseResponse {
        files,
        project,
        version,
    };
    debug!(?body, "Returning browse response");
    Response::builder().status(StatusCode::OK).json_body(&body)
}

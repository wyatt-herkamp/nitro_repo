use axum::{
    extract::{Path, Query, State},
    response::{IntoResponse, Response},
    routing::get,
};
use chrono::{DateTime, FixedOffset};
use nr_core::{repository::project::ProjectResolution, storage::StoragePath};
use nr_storage::{
    DirectoryFileType, FileFileType, FileHashes, FileType, SerdeMime, Storage, StorageFile,
    StorageFileMeta,
};
use serde::{Deserialize, Serialize};
use tracing::{debug, error, event, instrument, Level};
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

use crate::{
    app::{
        authentication::Authentication,
        responses::{MissingPermission, RepositoryNotFound},
        NitroRepo,
    },
    error::InternalError,
    repository::{utils::can_read_repository, Repository},
    utils::response_builder::ResponseBuilder,
};
pub fn browse_routes() -> axum::Router<NitroRepo> {
    axum::Router::new()
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
impl From<StorageFileMeta<FileType>> for BrowseFile {
    fn from(meta: StorageFileMeta<FileType>) -> Self {
        let StorageFileMeta {
            name,
            file_type,
            modified,
            created,
            ..
        } = meta;
        match file_type {
            FileType::File(FileFileType {
                file_size,
                mime_type,
                file_hash,
            }) => BrowseFile::File {
                name,
                file_size,
                mime_type,
                file_hash,
                modified,
                created,
            },
            FileType::Directory(DirectoryFileType { file_count }) => BrowseFile::Directory {
                name,
                number_of_files: file_count as usize,
            },
        }
    }
}
impl From<StorageFileMeta<FileFileType>> for BrowseFile {
    fn from(meta: StorageFileMeta<FileFileType>) -> Self {
        let StorageFileMeta {
            name,
            file_type,
            modified,
            created,
            ..
        } = meta;
        BrowseFile::File {
            name,
            file_size: file_type.file_size,
            mime_type: file_type.mime_type,
            file_hash: file_type.file_hash,
            modified,
            created,
        }
    }
}
#[derive(Debug, Serialize, Clone, ToSchema)]
pub struct BrowseResponse {
    pub files: Vec<BrowseFile>,
    /// The project contained in the path
    ///
    /// None if the check_for_project is false
    pub project_resolution: Option<ProjectResolution>,
}
#[utoipa::path(
    get,
    path = "/browse/{repository_id}/{path}",
    params(
        BrowseParams
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

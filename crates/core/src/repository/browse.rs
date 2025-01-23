use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::storage::{FileHashes, SerdeMime};

use super::project::ProjectResolution;

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
#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct BrowseResponse {
    pub files: Vec<BrowseFile>,
    /// The project contained in the path
    ///
    /// None if the check_for_project is false
    pub project_resolution: Option<ProjectResolution>,
}

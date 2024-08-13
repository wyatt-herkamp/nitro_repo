use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::repository::project::{ReleaseType, VersionData};
#[derive(Debug, Clone, PartialEq, Eq, Builder)]
pub struct NewProject {
    pub scope: Option<String>,
    /// Maven will use something like `{groupId}:{artifactId}`
    /// Cargo will use the `name` field
    pub project_key: String,
    /// Name of the project
    ///
    /// Maven will use the artifactId
    /// Cargo will use the `name` field
    /// NPM will use the `name` field
    pub name: String,
    /// Latest stable release
    pub latest_release: Option<String>,
    /// Release is SNAPSHOT in Maven or Alpha, Beta, on any other repository type
    /// This is the latest release or pre-release
    pub latest_pre_release: Option<String>,
    /// A short description of the project
    pub description: Option<String>,
    /// Can be empty
    pub tags: Vec<String>,
    /// The repository it belongs to
    pub repository: Uuid,
    /// Storage Path
    pub storage_path: String,
}
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
pub struct NewProjectMember {
    pub user_id: i32,
    pub project_id: Uuid,
    pub can_write: bool,
    pub can_manage: bool,
}
impl NewProjectMember {
    pub fn new_owner(user_id: i32, project: Uuid) -> Self {
        Self {
            user_id,
            project_id: project,
            can_write: true,
            can_manage: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Builder)]
pub struct NewVersion {
    pub project_id: Uuid,
    /// The version of the project
    pub version: String,
    /// Release type
    pub release_type: ReleaseType,
    /// The path to the release
    pub version_path: String,
    /// The publisher of the version
    pub publisher: i32,
    /// The version page. Such as a README
    pub version_page: Option<String>,
    /// The version data. More data can be added in the future and the data can be repository dependent
    pub extra: VersionData,
}

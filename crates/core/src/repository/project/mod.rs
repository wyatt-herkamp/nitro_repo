use pg_extended_sqlx_queries::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::prelude::Type;
use strum::{Display, EnumIs, EnumString, IntoStaticStr};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::database::entities::project::ProjectIds;
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema, Default)]
pub struct ProjectResolution {
    pub project_id: Option<Uuid>,
    pub version_id: Option<Uuid>,
}
impl From<ProjectIds> for ProjectResolution {
    fn from(ids: ProjectIds) -> Self {
        ProjectResolution {
            project_id: Some(ids.project_id),
            version_id: Some(ids.version_id),
        }
    }
}
/// Release type of a project
///
/// Can be overridden in the panel.
#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    Hash,
    Serialize,
    Deserialize,
    ToSchema,
    EnumIs,
    IntoStaticStr,
    Display,
    EnumString,
    Type,
    ValueExprType,
)]
#[sqlx(type_name = "TEXT")]
#[derive(Default)]
pub enum ReleaseType {
    /// Stable Release
    Stable,
    /// Beta Release
    Beta,
    /// Alpha Release
    Alpha,
    /// Snapshot Release
    /// Only really used in Maven
    Snapshot,
    /// .RC Release
    ReleaseCandidate,
    /// The release type could not be determined
    #[default]
    Unknown,
}

impl ReleaseType {
    pub fn release_type_from_version(version: &str) -> ReleaseType {
        let version = version.to_lowercase();
        if version.contains("snapshot") {
            ReleaseType::Snapshot
        } else if version.contains("beta") {
            ReleaseType::Beta
        } else if version.contains("alpha") {
            ReleaseType::Alpha
        } else if version.contains(".rc") {
            ReleaseType::ReleaseCandidate
        } else {
            ReleaseType::Stable
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema, sqlx::Type)]
pub enum ProjectState {
    Active,
    Deprecated,
}
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema, Default)]
#[serde(default)]

pub struct VersionData {
    pub documentation_url: Option<String>,
    pub website: Option<String>,
    #[serde(default)]
    pub authors: Vec<Author>,
    pub description: Option<String>,
    pub source: Option<ProjectSource>,
    pub licence: Option<Licence>,
    pub extra: Option<Value>,
}
/// Author of the project
///
/// All data is optional as artifact types may not have all the data
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
pub struct Author {
    /// Name of the author
    pub name: Option<String>,
    pub email: Option<String>,
    pub website: Option<String>,
}
/// Source of the project
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
#[serde(tag = "type", content = "value")]
pub enum ProjectSource {
    /// A git repository
    Git {
        url: String,
        branch: Option<String>,
        commit: Option<String>,
    },
}
/// Licence of the project Two Different types depending on how the artifact is setup
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
#[serde(tag = "type", content = "value")]
pub enum Licence {
    Simple(String),
    Array(Vec<LicenceValue>),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
pub struct LicenceValue {
    /// Licence Name
    pub name: String,
    /// Licence URL
    pub url: Option<String>,
}

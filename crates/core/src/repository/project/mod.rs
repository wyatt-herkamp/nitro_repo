use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
/// Release type of a project
///
/// Can be overridden in the panel.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema, sqlx::Type)]
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
    /// The release type could not be determined
    Unknown,
}
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema, sqlx::Type)]
pub enum ProjectState {
    Active,
    Deprecated,
}
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema, Default, Builder)]
#[serde(default)]
pub struct VersionData {
    pub documentation_url: Option<String>,
    pub website: Option<String>,
    #[serde(default)]
    pub authors: Vec<Author>,
    pub description: Option<String>,
    pub source: Option<ProjectSource>,
    pub licence: Option<Licence>,
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
    Git { url: String },
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

use std::str::FromStr;

use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgValueRef, prelude::Type, Decode, Encode};
use strum::{Display, EnumIs, EnumString, IntoStaticStr};
use utoipa::ToSchema;
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
)]
#[sqlx(type_name = "TEXT")]
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
    Unknown,
}

impl Default for ReleaseType {
    fn default() -> Self {
        ReleaseType::Unknown
    }
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
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema, Default, Builder)]
#[serde(default)]
pub struct VersionData {
    #[builder(default)]
    pub documentation_url: Option<String>,
    #[builder(default)]
    pub website: Option<String>,
    #[serde(default)]
    #[builder(default)]
    pub authors: Vec<Author>,
    #[builder(default)]
    pub description: Option<String>,
    #[builder(default)]
    pub source: Option<ProjectSource>,
    #[builder(default)]
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

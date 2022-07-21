use crate::repository::maven::staging::{DeployRequirement, StageSettings};
use serde::{Deserialize, Serialize};

use crate::repository::nitro::VersionData;
use crate::repository::settings::RepositoryConfigType;
use crate::utils::get_current_time;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DeployMetadata {
    #[serde(rename = "groupId")]
    pub group_id: String,
    #[serde(rename = "artifactId")]
    pub artifact_id: String,
    pub versioning: Versioning,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Versioning {
    pub release: Option<String>,
    pub versions: Versions,
    #[serde(rename = "lastUpdated")]
    pub last_updated: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Versions {
    pub version: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Scm {
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Pom {
    #[serde(rename = "groupId")]
    pub group_id: String,
    #[serde(rename = "artifactId")]
    pub artifact_id: String,
    pub version: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub url: Option<String>,
    pub scm: Option<Scm>,
}

impl Into<VersionData> for Pom {
    fn into(self) -> VersionData {
        VersionData {
            name: format!("{}:{}", &self.group_id, &self.artifact_id),
            description: self.description.unwrap_or_default(),
            source: None,
            licence: None,
            version: self.version,
            created: get_current_time(),
        }
    }
}

use crate::repository::data::RepositorySetting;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct MavenSettings {}

impl TryFrom<Value> for MavenSettings {
    type Error = serde_json::Error;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        serde_json::from_value(value)
    }
}

impl RepositorySetting for MavenSettings {}
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

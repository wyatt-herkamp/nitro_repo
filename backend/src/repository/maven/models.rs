use crate::repository::maven::staging::{DeployRequirement, StageSettings};
use serde::{Deserialize, Serialize};

use crate::repository::nitro::VersionData;
use crate::repository::settings::RepositoryConfigType;
use crate::utils::get_current_time;

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct MavenSettings {
    pub repository_type: MavenType,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "page_type", content = "content")]
pub enum MavenType {
    /// Hosted on the Storage Only
    Hosted { allow_pushing: bool },
    ///  An intermediary repository for intake of new artifacts. Pushes the artifacts to other repositories
    Staging {
        stage_to: Vec<StageSettings>,
        pre_stage_requirements: Vec<DeployRequirement>,
    },
    /// Uses Remote Proxies to get the artifacts.
    /// Uses the storage to hold a backup of the artifacts.
    Proxy { proxies: Vec<ProxySettings> },
}
impl Default for MavenType {
    fn default() -> Self {
        MavenType::Hosted {
            allow_pushing: true,
        }
    }
}
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct ProxySettings {
    pub proxy_url: String,
    pub proxy_username: Option<LoginSettings>,
}
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct LoginSettings {
    pub username: String,
    pub password: String,
}

impl RepositoryConfigType for MavenSettings {
    fn config_name() -> &'static str {
        "maven.json"
    }
}

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

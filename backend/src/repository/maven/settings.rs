use crate::repository::maven::staging::{DeployRequirement, StageSettings};
use crate::repository::settings::RepositoryConfigType;
use serde::{Deserialize, Serialize};

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
        /// This is a parent that nothing is actually pushed it. It just allows for data retrieval.
        parent: ProxySettings,
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
    pub login: Option<LoginSettings>,
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

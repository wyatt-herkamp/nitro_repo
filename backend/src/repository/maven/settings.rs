use crate::repository::maven::staging::{DeployRequirement, StageSettings};
use crate::repository::settings::RepositoryConfigType;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct MavenSettings {
    pub repository_type: MavenType,
}
impl RepositoryConfigType for MavenSettings {
    fn config_name() -> &'static str {
        "maven.json"
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub enum MavenType {
    #[default]
    /// Hosted on the Storage Only
    Hosted,
    ///  An intermediary repository for intake of new artifacts. Pushes the artifacts to other repositories
    Staging,
    /// Uses Remote Proxies to get the artifacts.
    /// Uses the storage to hold a backup of the artifacts.
    Proxy,
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

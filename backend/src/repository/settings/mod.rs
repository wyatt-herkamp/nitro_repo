use crate::repository::settings::frontend::{BadgeSettings, Frontend};
use serde::{Serialize, Deserialize};

pub mod security;
pub mod frontend;
pub mod webhook;

fn default() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize, strum_macros::EnumString)]
pub enum Policy {
    Release,
    Snapshot,
    Mixed,
}

impl Default for Policy {
    fn default() -> Self {
        Policy::Mixed
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct RepositorySettings {
    #[serde(default = "default")]
    pub active: bool,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub policy: Policy,
    #[serde(default)]
    pub frontend: Frontend,
    #[serde(default )]
    pub badge: BadgeSettings,
}

impl Default for RepositorySettings {
    fn default() -> Self {
        RepositorySettings {
            active: true,
            description: "".to_string(),
            policy: Policy::Mixed,
            frontend: Default::default(),
            badge: Default::default(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateSettings {
    pub active: bool,
    pub policy: Policy,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateFrontend {
    pub frontend: Frontend,
    pub badge: BadgeSettings,
}


impl RepositorySettings {
    pub fn update_general(&mut self, settings: UpdateSettings) {
        self.policy = settings.policy;
        self.active = settings.active;
    }
    pub fn update_frontend(&mut self, settings: UpdateFrontend) {
        self.frontend = settings.frontend;
        self.badge = settings.badge;
    }
}

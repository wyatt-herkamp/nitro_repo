use crate::repository::settings::frontend::{BadgeSettings, Frontend};
use serde::{Deserialize, Serialize};

pub mod frontend;
pub mod security;
pub mod webhook;


#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, strum_macros::EnumString)]
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
    #[serde(default)]
    pub frontend: Frontend,
    #[serde(default)]
    pub badge: BadgeSettings,
}



#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateFrontend {
    pub frontend: Frontend,
    pub badge: BadgeSettings,
}

impl RepositorySettings {

    pub fn update_frontend(&mut self, settings: UpdateFrontend) {
        self.frontend = settings.frontend;
        self.badge = settings.badge;
    }
}

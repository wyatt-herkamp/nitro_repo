use serde::{Deserialize, Serialize};

pub mod frontend;
pub mod security;
pub mod webhook;

pub const WEBHOOK_CONFIG: &str = ".nitro_repo/webhook.json";
pub const FRONTEND_CONFIG: &str = ".nitro_repo/frontend.json";

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

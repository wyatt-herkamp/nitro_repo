use serde::{Deserialize, Serialize};

use utoipa::Component;

#[derive(Debug, Clone, Serialize, Deserialize, Component)]
pub struct ProjectResponse {}

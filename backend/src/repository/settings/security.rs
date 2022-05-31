use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, strum_macros::EnumString)]
pub enum Visibility {
    Public,
    Private,
    Hidden,
}

impl Default for Visibility {
    fn default() -> Self {
        Visibility::Public
    }
}

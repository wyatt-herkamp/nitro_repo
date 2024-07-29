use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use strum::EnumIter;

pub mod config;
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default, EnumIter, JsonSchema)]
pub enum Visibility {
    #[default]
    Public,
    Private,
    Hidden,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default, EnumIter, JsonSchema)]
pub enum Policy {
    #[default]
    Release,
    Snapshot,
    Mixed,
}

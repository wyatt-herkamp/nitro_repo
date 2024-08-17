use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use strum::{EnumIs, EnumIter};
pub mod config;
pub mod project;
#[derive(
    Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default, EnumIter, JsonSchema, EnumIs,
)]
pub enum Visibility {
    /// Completely public anyone can read to this repository
    #[default]
    Public,
    /// Private. Only users with the correct permissions can read this repository
    Private,
    /// Hidden. You can read this repository but indexing will be disabled
    Hidden,
}
#[derive(
    Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default, EnumIter, JsonSchema, EnumIs,
)]
pub enum Policy {
    #[default]
    Release,
    Snapshot,
    Mixed,
}
#[derive(
    Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default, EnumIter, JsonSchema, EnumIs,
)]
pub enum Test {
    #[default]
    Public,
    Private,
    Hidden,
}

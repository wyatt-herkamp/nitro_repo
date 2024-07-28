use serde::{Deserialize, Serialize};
use strum::EnumIter;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default, EnumIter, sqlx::Type)]
pub enum Visibility {
    #[default]
    Public,
    Private,
    Hidden,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default, EnumIter)]
pub enum Policy {
    #[default]
    Release,
    Snapshot,
    Mixed,
}

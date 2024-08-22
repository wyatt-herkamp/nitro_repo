use serde::{Deserialize, Serialize};
use sqlx::prelude::Type;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Type, Serialize, Deserialize)]
#[sqlx(type_name = "TEXT")]
pub enum Scopes {
    ReadRepository,
    WriteRepository,
    EditRepository,
}

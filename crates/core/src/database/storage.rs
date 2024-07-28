use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use super::DateTime;

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]

pub struct DBStorage {
    pub id: Uuid,
    pub storage_type: String,
    pub name: String,
    pub config: Value,
    pub active: bool,
    pub created: DateTime,
}

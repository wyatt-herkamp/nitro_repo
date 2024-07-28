use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::types::Json;
use uuid::Uuid;

use crate::repository::Visibility;

use super::DateTime;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DBRepository {
    pub id: Uuid,
    pub storage_id: Uuid,
    pub name: String,
    pub visibility: Visibility,
    pub active: bool,
    pub created: DateTime,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct DBRepositoryConfig<T> {
    pub id: i64,
    /// The Repository ID that this config belongs to.
    pub repository_id: Uuid,
    /// The Repository Config Key. This is the key that used to identify a config type.
    pub key: String,
    /// The Repository Config Value. This is the value that is stored for the key.
    pub value: Json<T>,
    pub updated: DateTime,
    pub created: DateTime,
}
pub type GenericDBRepositoryConfig = DBRepositoryConfig<Value>;

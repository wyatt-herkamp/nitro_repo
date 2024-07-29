use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::{prelude::FromRow, types::Json};
use uuid::Uuid;

use super::DateTime;

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize, FromRow)]

pub struct DBStorage {
    pub id: Uuid,
    pub storage_type: String,
    pub name: String,
    pub config: Json<Value>,
    pub active: bool,
    pub created: DateTime,
}
impl DBStorage {
    pub async fn get_all(database: &sqlx::PgPool) -> Result<Vec<Self>, sqlx::Error> {
        let storages = sqlx::query_as("SELECT * FROM storages")
            .fetch_all(database)
            .await?;
        Ok(storages)
    }
}

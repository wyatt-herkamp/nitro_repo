use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::{prelude::FromRow, types::Json};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::storage::StorageName;

use super::DateTime;
pub struct NewDBStorage {
    pub storage_type: String,
    pub name: StorageName,
    pub config: Json<Value>,
}
impl NewDBStorage {
    pub fn new(storage_type: String, name: StorageName, config: Value) -> Self {
        Self {
            storage_type,
            name,
            config: Json(config),
        }
    }
    pub async fn insert(self, database: &sqlx::PgPool) -> Result<Option<DBStorage>, sqlx::Error> {
        let result = sqlx::query_as(
            r#"INSERT INTO storages (storage_type, name, config) VALUES ($1, $2, $3) ON CONFLICT (name) DO NOTHING RETURNING *"#,
        )
        .bind(self.storage_type)
        .bind(self.name)
        .bind(self.config)
        .fetch_optional(database)
        .await?;
        Ok(result)
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize, FromRow, ToSchema)]

pub struct DBStorage {
    pub id: Uuid,
    pub storage_type: String,
    pub name: StorageName,
    /// The configuration for the storage
    /// This is based on the storage type. It is stored as a JSON object.
    ///  Requests should be JSON and the response will be JSON. Please refer to the storage type documentation for the configuration.
    pub config: Json<Value>,
    pub active: bool,
    pub updated_at: DateTime,
    pub created_at: DateTime,
}
impl DBStorage {
    pub async fn get_all(database: &sqlx::PgPool) -> Result<Vec<Self>, sqlx::Error> {
        let storages = sqlx::query_as("SELECT * FROM storages")
            .fetch_all(database)
            .await?;
        Ok(storages)
    }
    pub async fn get(id: Uuid, database: &sqlx::PgPool) -> Result<Option<Self>, sqlx::Error> {
        let storage = sqlx::query_as("SELECT * FROM storages WHERE id = $1")
            .bind(id)
            .fetch_optional(database)
            .await?;
        Ok(storage)
    }

    pub async fn delete(id: Uuid, database: &sqlx::PgPool) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM storages WHERE id = $1")
            .bind(id)
            .execute(database)
            .await?;
        Ok(())
    }
    pub async fn is_name_available(
        name: &str,
        database: &sqlx::PgPool,
    ) -> Result<bool, sqlx::Error> {
        let result: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM storages WHERE name = $1;")
            .bind(name)
            .fetch_one(database)
            .await?;
        Ok(result == 0)
    }
}

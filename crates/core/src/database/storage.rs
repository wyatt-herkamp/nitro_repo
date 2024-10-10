use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::{postgres::PgRow, prelude::FromRow, types::Json};
use tracing::{instrument, trace};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::storage::StorageName;

use super::DateTime;

pub trait StorageDBType: for<'r> FromRow<'r, PgRow> + Unpin + Send + Sync {
    fn columns() -> Vec<&'static str>;
    fn format_columns(prefix: Option<&str>) -> String {
        if let Some(prefix) = prefix {
            Self::columns()
                .iter()
                .map(|column| format!("{}.`{}`", prefix, column))
                .collect::<Vec<String>>()
                .join(", ")
        } else {
            Self::columns().join(", ")
        }
    }
    fn id(&self) -> Uuid;
    #[instrument(name = "get_all_storages", skip(database))]
    async fn get_all(database: &sqlx::PgPool) -> Result<Vec<Self>, sqlx::Error> {
        let columns = Self::format_columns(None);
        let query = format!("SELECT {} FROM storages", columns);
        trace!(?query);
        let storages = sqlx::query_as(&query).fetch_all(database).await?;
        Ok(storages)
    }
    #[instrument(name = "get_active_storages", skip(database))]
    async fn get_all_active(database: &sqlx::PgPool) -> Result<Vec<Self>, sqlx::Error> {
        let columns = Self::format_columns(None);
        let query = format!("SELECT {} FROM storages WHERE active = true", columns);
        trace!(?query);
        let storages = sqlx::query_as(&query).fetch_all(database).await?;
        Ok(storages)
    }
    #[instrument(name = "get_by_id", skip(database))]
    async fn get_by_id(id: Uuid, database: &sqlx::PgPool) -> Result<Option<Self>, sqlx::Error> {
        let columns = Self::format_columns(None);
        let query = format!("SELECT {} FROM storages WHERE id = $1", columns);
        trace!(?query);
        let storage = sqlx::query_as(&query)
            .bind(id)
            .fetch_optional(database)
            .await?;
        Ok(storage)
    }
    async fn delete_self(&self, database: &sqlx::PgPool) -> Result<(), sqlx::Error> {
        let query = "DELETE FROM storages WHERE id = $1".to_string();
        sqlx::query(&query)
            .bind(self.id())
            .execute(database)
            .await?;
        Ok(())
    }
}

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
pub struct DBStorageNoConfig {
    pub id: Uuid,
    pub storage_type: String,
    pub name: StorageName,
    pub active: bool,
    pub updated_at: DateTime,
    pub created_at: DateTime,
}
impl StorageDBType for DBStorageNoConfig {
    fn id(&self) -> Uuid {
        self.id
    }
    fn columns() -> Vec<&'static str> {
        vec![
            "id",
            "storage_type",
            "name",
            "active",
            "updated_at",
            "created_at",
        ]
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
    #[schema(value_type = crate::utils::utopia::AnyType)]
    pub config: Json<Value>,
    pub active: bool,
    pub updated_at: DateTime,
    pub created_at: DateTime,
}

impl StorageDBType for DBStorage {
    fn columns() -> Vec<&'static str> {
        vec![
            "id",
            "storage_type",
            "name",
            "config",
            "active",
            "updated_at",
            "created_at",
        ]
    }
    fn id(&self) -> Uuid {
        self.id
    }
}
impl DBStorage {
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

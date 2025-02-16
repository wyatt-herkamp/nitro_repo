use crate::database::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::{postgres::PgRow, prelude::FromRow, types::Json};
use tracing::instrument;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::storage::StorageName;

pub trait StorageDBType:
    for<'r> FromRow<'r, PgRow> + Unpin + Send + Sync + TableQuery<Table = DBStorage>
{
    fn id(&self) -> Uuid;
    #[instrument(name = "get_all_storages", skip(database))]
    async fn get_all(database: &sqlx::PgPool) -> Result<Vec<Self>, sqlx::Error> {
        let storages = SelectQueryBuilder::with_columns(DBStorage::table_name(), Self::columns())
            .query_as()
            .fetch_all(database)
            .await?;
        Ok(storages)
    }
    #[instrument(name = "get_active_storages", skip(database))]
    async fn get_all_active(database: &sqlx::PgPool) -> Result<Vec<Self>, sqlx::Error> {
        let storages = SelectQueryBuilder::with_columns(DBStorage::table_name(), Self::columns())
            .filter(DBStorageColumn::Active.equals(true.value()))
            .query_as()
            .fetch_all(database)
            .await?;

        Ok(storages)
    }
    #[instrument(name = "get_by_id", skip(database))]
    async fn get_by_id(id: Uuid, database: &sqlx::PgPool) -> Result<Option<Self>, sqlx::Error> {
        let storage = SelectQueryBuilder::with_columns(DBStorage::table_name(), Self::columns())
            .filter(DBStorageColumn::Id.equals(id.value()))
            .query_as()
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
    pub updated_at: chrono::DateTime<chrono::FixedOffset>,
    pub created_at: chrono::DateTime<chrono::FixedOffset>,
}
impl TableQuery for DBStorageNoConfig {
    type Table = DBStorage;

    fn columns() -> Vec<<Self::Table as TableType>::Columns>
    where
        Self: Sized,
    {
        DBStorage::columns()
    }
}
impl StorageDBType for DBStorageNoConfig {
    fn id(&self) -> Uuid {
        self.id
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize, FromRow, ToSchema, TableType)]
#[table(name = "storages")]
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
    pub updated_at: chrono::DateTime<chrono::FixedOffset>,
    pub created_at: chrono::DateTime<chrono::FixedOffset>,
}

impl StorageDBType for DBStorage {
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

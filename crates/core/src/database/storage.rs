use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::{prelude::FromRow, types::Json};
use utoipa::ToSchema;
use uuid::Uuid;

use super::DateTime;

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize, FromRow, ToSchema)]

pub struct DBStorage {
    pub id: Uuid,
    pub storage_type: String,
    pub name: String,
    /// The configuration for the storage
    /// This is based on the storage type. It is stored as a JSON object.
    ///  Requests should be JSON and the response will be JSON. Please refer to the storage type documentation for the configuration.
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
    pub async fn get(id: Uuid, database: &sqlx::PgPool) -> Result<Option<Self>, sqlx::Error> {
        let storage = sqlx::query_as("SELECT * FROM storages WHERE id = $1")
            .bind(id)
            .fetch_optional(database)
            .await?;
        Ok(storage)
    }
    pub async fn insert(self, database: &sqlx::PgPool) -> Result<DBStorage, sqlx::Error> {
        let result = sqlx::query_as(
                    r#"INSERT INTO storages (id, storage_type, name, config, active) VALUES ($1, $2, $3, $4, $5) RETURNING *"#,
                )
                .bind(&self.id)
                .bind(&self.storage_type)
                .bind(&self.name)
                .bind(&self.config)
                .bind(&self.active)
                .fetch_one(database).await?;
        Ok(result)
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
        let result: i64 = sqlx::query_scalar("SELECT COUNT(*) storages WHERE name = $1;")
            .bind(name)
            .fetch_one(database)
            .await?;
        Ok(result == 0)
    }
    pub async fn generate_uuid(database: &sqlx::PgPool) -> Result<Uuid, sqlx::Error> {
        let mut uuid = Uuid::new_v4();
        while sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM storages WHERE id = $1;")
            .bind(&uuid)
            .fetch_one(database)
            .await?
            > 0
        {
            uuid = Uuid::new_v4();
        }
        Ok(uuid)
    }
}

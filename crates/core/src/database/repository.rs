use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::{prelude::FromRow, types::Json, PgPool};
use uuid::Uuid;

use crate::repository::Visibility;

use super::DateTime;
#[derive(Debug, Clone, Serialize, FromRow)]

pub struct DBRepositoryWithStorageName {
    pub id: Uuid,
    pub storage_id: Uuid,
    pub storage_name: String,
    pub name: String,
    pub repository_type: String,
    pub repository_subtype: Option<String>,
    pub active: bool,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}
impl DBRepositoryWithStorageName {
    pub async fn get_all(database: &PgPool) -> Result<Vec<Self>, sqlx::Error> {
        let repositories = sqlx::query_as(
            r#"SELECT r.id, r.storage_id, s.name AS storage_name, r.name, r.repository_type, r.repository_subtype, r.active, r.created_at, r.updated_at
                FROM repositories r INNER JOIN storages s ON s.id = r.storage_id"#,
        )
        .fetch_all(database)
        .await?;
        Ok(repositories)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, FromRow)]
pub struct DBRepository {
    pub id: Uuid,
    pub storage_id: Uuid,
    pub name: String,
    pub repository_type: String,
    pub repository_subtype: Option<String>,
    pub active: bool,
    pub created: DateTime,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, FromRow)]
pub struct RepositoryLookup {
    pub storage_id: Uuid,
    pub repository_id: Uuid,
}
impl DBRepository {
    pub async fn get_id_from_storage_and_name(
        storage_name: impl AsRef<str>,
        repository_name: impl AsRef<str>,
        database: &PgPool,
    ) -> Result<Option<RepositoryLookup>, sqlx::Error> {
        let id = sqlx::query_as(
            r#"SELECT r.id AS repository_id, s.id AS storage_id FROM repositories r FULL OUTER JOIN storages s ON s.id = r.storage_id WHERE s.name = $1 AND r.name = $2"#,
        )
        .bind(storage_name.as_ref())
        .bind(repository_name.as_ref())
        .fetch_optional(database)
        .await?;
        Ok(id)
    }
    pub async fn get_by_id(id: Uuid, database: &PgPool) -> Result<Option<Self>, sqlx::Error> {
        let repository =
            sqlx::query_as::<_, DBRepository>("SELECT * FROM repositories WHERE id = $1")
                .bind(id)
                .fetch_optional(database)
                .await?;
        Ok(repository)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize, FromRow, Default)]
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
impl DBRepositoryConfig<Value> {
    pub async fn add_or_update(
        uuid: Uuid,
        key: String,
        value: Value,
        database: &PgPool,
    ) -> Result<(), sqlx::Error> {
        // Check if the config already exists
        let config = sqlx::query_as::<_, DBRepositoryConfig<Value>>(
            r#"SELECT * FROM repository_configs WHERE repository_id = $1 AND key = $2"#,
        )
        .bind(uuid)
        .bind(&key)
        .fetch_optional(database)
        .await?;
        if let Some(config) = config {
            sqlx::query(
                r#"UPDATE repository_configs SET value = $1, updated = NOW() WHERE id = $2"#,
            )
            .bind(Json(value))
            .bind(config.id)
            .execute(database)
            .await?;
        } else {
            sqlx::query(
                r#"INSERT INTO repository_configs (repository_id, key, value) VALUES ($1, $2, $3)"#,
            )
            .bind(uuid)
            .bind(&key)
            .bind(Json(value))
            .execute(database)
            .await?;
        }
        Ok(())
    }
}

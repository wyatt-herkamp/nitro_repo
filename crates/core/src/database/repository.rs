use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::{postgres::PgRow, prelude::FromRow, types::Json, PgPool};
use tracing::info;
use utoipa::ToSchema;
use uuid::Uuid;
mod hostname;
use crate::{
    repository::{RepositoryName, Visibility},
    storage::StorageName,
};
pub use hostname::*;

use super::DateTime;
pub trait RepositoryDBType: for<'r> FromRow<'r, PgRow> + Unpin + Send + Sync {
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
}
#[derive(Debug, Clone, Serialize, FromRow, ToSchema)]

pub struct DBRepositoryWithStorageName {
    pub id: Uuid,
    pub storage_id: Uuid,
    pub storage_name: StorageName,
    pub name: RepositoryName,
    pub repository_type: String,
    pub visibility: Visibility,
    pub active: bool,
    pub updated_at: DateTime,
    pub created_at: DateTime,
}
impl DBRepositoryWithStorageName {
    pub async fn get_all(database: &PgPool) -> Result<Vec<Self>, sqlx::Error> {
        let repositories = sqlx::query_as(
            r#"SELECT r.id, r.storage_id, s.name AS storage_name, r.name, r.repository_type, r.visibility, r.active, r.created_at, r.updated_at
                FROM repositories r INNER JOIN storages s ON s.id = r.storage_id"#,
        )
        .fetch_all(database)
        .await?;
        Ok(repositories)
    }
    pub async fn get_by_id(id: Uuid, database: &PgPool) -> Result<Option<Self>, sqlx::Error> {
        let repository = sqlx::query_as(
            r#"SELECT r.id, r.storage_id, s.name AS storage_name, r.name, r.repository_type, r.visibility, r.active, r.created_at, r.updated_at
                FROM repositories r INNER JOIN storages s ON s.id = r.storage_id WHERE r.id = $1"#,
        )
        .bind(id)
        .fetch_optional(database)
        .await?;
        Ok(repository)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, FromRow, ToSchema)]
pub struct DBRepository {
    pub id: Uuid,
    pub storage_id: Uuid,
    pub name: RepositoryName,
    pub repository_type: String,
    pub visibility: Visibility,
    pub active: bool,
    pub updated_at: DateTime,
    pub created_at: DateTime,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, FromRow)]
pub struct RepositoryLookup {
    pub storage_id: Uuid,
    pub repository_id: Uuid,
}
impl DBRepository {
    pub async fn get_active_by_id(
        id: Uuid,
        database: &PgPool,
    ) -> Result<Option<bool>, sqlx::Error> {
        let is_active = sqlx::query_scalar(r#"SELECT active FROM repositories WHERE id = $1"#)
            .bind(id)
            .fetch_optional(database)
            .await?;
        Ok(is_active)
    }
    pub async fn get_all(database: &PgPool) -> Result<Vec<Self>, sqlx::Error> {
        let repositories = sqlx::query_as("SELECT * FROM repositories")
            .fetch_all(database)
            .await?;
        Ok(repositories)
    }
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

    pub async fn does_name_exist_for_storage(
        storage_id: Uuid,
        name: impl AsRef<str>,
        database: &PgPool,
    ) -> Result<bool, sqlx::Error> {
        let result: i64 = sqlx::query_scalar(
            r#"SELECT COUNT(*) FROM repositories WHERE storage_id = $1 AND name = $2"#,
        )
        .bind(storage_id)
        .bind(name.as_ref())
        .fetch_one(database)
        .await?;
        info!(
            "Found {} repositories with the name {} for storage {}",
            result,
            name.as_ref(),
            storage_id
        );
        Ok(result != 0)
    }
    pub async fn generate_uuid(database: &PgPool) -> Result<Uuid, sqlx::Error> {
        let mut uuid = Uuid::new_v4();
        while sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM storages WHERE id = $1;")
            .bind(uuid)
            .fetch_one(database)
            .await?
            > 0
        {
            uuid = Uuid::new_v4();
        }
        Ok(uuid)
    }
    pub async fn delete_by_id(id: Uuid, database: &PgPool) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM repositories WHERE id = $1")
            .bind(id)
            .execute(database)
            .await?;
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize, FromRow, Default)]
pub struct DBRepositoryConfig<T> {
    pub id: i32,
    /// The Repository ID that this config belongs to.
    pub repository_id: Uuid,
    /// The Repository Config Key. This is the key that used to identify a config type.
    pub key: String,
    /// The Repository Config Value. This is the value that is stored for the key.
    pub value: Json<T>,
    pub updated_at: DateTime,
    pub created_at: DateTime,
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
                r#"UPDATE repository_configs SET value = $1, updated_at = NOW() WHERE id = $2"#,
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

impl<T: for<'a> Deserialize<'a> + Unpin + Send + 'static> DBRepositoryConfig<T> {
    pub async fn get_config(
        uuid: Uuid,
        key: impl AsRef<str>,
        database: &PgPool,
    ) -> Result<Option<Self>, sqlx::Error> {
        let config = sqlx::query_as::<_, DBRepositoryConfig<T>>(
            r#"SELECT * FROM repository_configs WHERE repository_id = $1 AND key = $2"#,
        )
        .bind(uuid)
        .bind(key.as_ref())
        .fetch_optional(database)
        .await?;
        Ok(config)
    }
}

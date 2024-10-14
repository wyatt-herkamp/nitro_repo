use serde::Serialize;
use sqlx::prelude::FromRow;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::database::DateTime;
/// A hostname that is associated with a repository.
///
/// Table: `repository_hostnames`

#[derive(Debug, Clone, Serialize, FromRow, ToSchema)]
pub struct DBRepositoryHostname {
    pub id: i32,
    pub repository_id: Uuid,
    pub hostname: String,
    pub updated_at: DateTime,
    pub created_at: DateTime,
}
impl DBRepositoryHostname {
    pub async fn is_hostname_available(
        database: &sqlx::PgPool,
        hostname: &str,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM repository_hostnames WHERE hostname = $1)",
        )
        .bind(hostname)
        .fetch_one(database)
        .await?;
        Ok(result)
    }
    pub async fn get_by_hostname(
        database: &sqlx::PgPool,
        hostname: &str,
    ) -> Result<Option<Self>, sqlx::Error> {
        let result = sqlx::query_as("SELECT * FROM repository_hostnames WHERE hostname = $1")
            .bind(hostname)
            .fetch_optional(database)
            .await?;
        Ok(result)
    }

    pub async fn get_by_repository_id(
        database: &sqlx::PgPool,
        repository_id: Uuid,
    ) -> Result<Vec<Self>, sqlx::Error> {
        let result = sqlx::query_as("SELECT * FROM repository_hostnames WHERE repository_id = $1")
            .bind(repository_id)
            .fetch_all(database)
            .await?;
        Ok(result)
    }
    pub async fn get_all(database: &sqlx::PgPool) -> Result<Vec<Self>, sqlx::Error> {
        let result = sqlx::query_as("SELECT * FROM repository_hostnames")
            .fetch_all(database)
            .await?;
        Ok(result)
    }
}

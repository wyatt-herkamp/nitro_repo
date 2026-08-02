use serde::Serialize;
use sqlx::{PgPool, prelude::FromRow};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{database::DateTime, repository::Hostname};

/// The columns of [`DBRepositoryHostname`], in declaration order.
///
/// Spelled out rather than `SELECT *` because the table is shared with storage-scoped hostnames
/// and carries a `storage_id` this type has no field for.
const COLUMNS: &str = "id, repository_id, hostname, updated_at, created_at";

/// A hostname that routes a request straight into a repository.
///
/// Table: `hostnames` — shared with storage-scoped hostnames, which have `repository_id` NULL and
/// `storage_id` set. Every query here filters those out, so the non-optional `repository_id` below
/// always decodes.
#[derive(Debug, Clone, Serialize, FromRow, ToSchema)]
pub struct DBRepositoryHostname {
    pub id: i32,
    pub repository_id: Uuid,
    pub hostname: Hostname,
    pub updated_at: DateTime,
    pub created_at: DateTime,
}
impl DBRepositoryHostname {
    /// Whether the hostname is already claimed — by a repository *or* a storage.
    ///
    /// Deliberately not filtered to repository rows: the unique index is global, so a
    /// storage-scoped row is a genuine conflict rather than something to route around.
    pub async fn is_hostname_taken(database: &PgPool, hostname: &str) -> Result<bool, sqlx::Error> {
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM hostnames WHERE hostname = $1)")
            .bind(hostname)
            .fetch_one(database)
            .await
    }

    pub async fn get_by_hostname(
        database: &PgPool,
        hostname: &str,
    ) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as(&format!(
            "SELECT {COLUMNS} FROM hostnames WHERE hostname = $1 AND repository_id IS NOT NULL"
        ))
        .bind(hostname)
        .fetch_optional(database)
        .await
    }

    pub async fn get_by_repository_id(
        database: &PgPool,
        repository_id: Uuid,
    ) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as(&format!(
            "SELECT {COLUMNS} FROM hostnames WHERE repository_id = $1 ORDER BY created_at"
        ))
        .bind(repository_id)
        .fetch_all(database)
        .await
    }

    pub async fn get_all(database: &PgPool) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as(&format!(
            "SELECT {COLUMNS} FROM hostnames WHERE repository_id IS NOT NULL ORDER BY created_at"
        ))
        .fetch_all(database)
        .await
    }

    /// Every `(hostname, repository_id)` pair, for building the in-memory routing index at startup.
    pub async fn all_pairs(database: &PgPool) -> Result<Vec<(String, Uuid)>, sqlx::Error> {
        sqlx::query_as(
            "SELECT hostname, repository_id FROM hostnames WHERE repository_id IS NOT NULL",
        )
        .fetch_all(database)
        .await
    }

    pub async fn insert(
        database: &PgPool,
        repository_id: Uuid,
        hostname: &Hostname,
    ) -> Result<Self, sqlx::Error> {
        sqlx::query_as(&format!(
            "INSERT INTO hostnames (repository_id, hostname) VALUES ($1, $2) RETURNING {COLUMNS}"
        ))
        .bind(repository_id)
        .bind(hostname)
        .fetch_one(database)
        .await
    }

    /// Deletes a hostname, scoped to the repository that owns it.
    ///
    /// Scoping by repository means an id from another repository is a 404 rather than a deletion,
    /// so the route cannot be used to unregister someone else's domain by guessing. Returns the
    /// deleted row so the caller knows which host to evict from the routing index.
    pub async fn delete(
        database: &PgPool,
        id: i32,
        repository_id: Uuid,
    ) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as(&format!(
            "DELETE FROM hostnames WHERE id = $1 AND repository_id = $2 RETURNING {COLUMNS}"
        ))
        .bind(id)
        .bind(repository_id)
        .fetch_optional(database)
        .await
    }
}

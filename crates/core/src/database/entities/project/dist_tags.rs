//! npm distribution tags.
//!
//! A dist-tag is a mutable pointer from a name like `latest`, `next` or `beta` to one version of a
//! project. npm resolves a bare `npm install pkg` through `latest`, so this is load bearing rather
//! than decorative.
//!
//! There was nowhere to store one before, so a packument's `dist-tags` only ever contained
//! `latest`, and that was synthesized from whichever version row the database returned first —
//! `npm publish --tag next` silently published to `latest`, and `npm dist-tag add` had no route at
//! all.
use serde::Serialize;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::database::prelude::*;

/// The tag npm resolves when none is named. Removing it would make `npm install pkg` fail, so the
/// delete path refuses.
pub const LATEST_TAG: &str = "latest";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, FromRow, ToSchema, TableType)]
#[table(name = "npm_dist_tags")]
pub struct DBNpmDistTag {
    pub project_id: Uuid,
    pub tag: String,
    pub version: String,
    pub updated_at: DateTime<FixedOffset>,
    pub created_at: DateTime<FixedOffset>,
}

impl DBNpmDistTag {
    /// Every tag for a project, as the `dist-tags` map a packument carries.
    #[instrument(skip(database))]
    pub async fn get_all_for_project(
        project_id: Uuid,
        database: &PgPool,
    ) -> Result<Vec<Self>, sqlx::Error> {
        SelectQueryBuilder::with_columns(Self::table_name(), Self::columns())
            .filter(DBNpmDistTagColumn::ProjectId.equals(project_id.value()))
            .order_by(DBNpmDistTagColumn::Tag, SQLOrder::Ascending)
            .query_as()
            .fetch_all(database)
            .await
    }

    #[instrument(skip(database))]
    pub async fn get(
        project_id: Uuid,
        tag: &str,
        database: &PgPool,
    ) -> Result<Option<Self>, sqlx::Error> {
        SelectQueryBuilder::with_columns(Self::table_name(), Self::columns())
            .filter(DBNpmDistTagColumn::ProjectId.equals(project_id.value()))
            .filter(DBNpmDistTagColumn::Tag.equals(tag.value()))
            .query_as()
            .fetch_optional(database)
            .await
    }

    /// Points a tag at a version, creating it if it does not exist.
    ///
    /// A plain read-then-write would let two concurrent publishes of the same package each see no
    /// row and both insert, so this is a single `ON CONFLICT` statement.
    #[instrument(skip(database))]
    pub async fn set(
        project_id: Uuid,
        tag: &str,
        version: &str,
        database: &PgPool,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"INSERT INTO npm_dist_tags (project_id, tag, version)
               VALUES ($1, $2, $3)
               ON CONFLICT (project_id, tag)
               DO UPDATE SET version = EXCLUDED.version, updated_at = CURRENT_TIMESTAMP"#,
        )
        .bind(project_id)
        .bind(tag)
        .bind(version)
        .execute(database)
        .await?;
        Ok(())
    }

    #[instrument(skip(database))]
    pub async fn delete(
        project_id: Uuid,
        tag: &str,
        database: &PgPool,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(r#"DELETE FROM npm_dist_tags WHERE project_id = $1 AND tag = $2"#)
            .bind(project_id)
            .bind(tag)
            .execute(database)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    /// Drops every tag pointing at a version that is going away.
    ///
    /// Without this an unpublish leaves `latest` aimed at a version whose tarball has been
    /// deleted, and `npm install` fails on a 404 rather than resolving to a version that exists.
    #[instrument(skip(database))]
    pub async fn delete_pointing_at(
        project_id: Uuid,
        version: &str,
        database: &PgPool,
    ) -> Result<Vec<String>, sqlx::Error> {
        let tags: Vec<String> = sqlx::query_scalar(
            r#"DELETE FROM npm_dist_tags WHERE project_id = $1 AND version = $2 RETURNING tag"#,
        )
        .bind(project_id)
        .bind(version)
        .fetch_all(database)
        .await?;
        Ok(tags)
    }
}

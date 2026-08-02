//! Manifests a Docker repository holds, addressed by digest.
//!
//! Tags are not here — a tag is a version, so it lives in `project_versions` with every other
//! repository type's versions. This table covers the two cases a version row cannot: a pull that
//! names a digest rather than a tag, and a manifest nothing is tagged at (the per-platform children
//! of an index, and OCI referrers artifacts), which must still be retained and served.

use serde::Serialize;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::database::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, FromRow, ToSchema, TableType)]
#[table(name = "docker_manifests")]
pub struct DBDockerManifest {
    pub id: Uuid,
    pub repository_id: Uuid,
    pub image_name: String,
    /// `algorithm:hex`, over the exact bytes stored.
    pub digest: String,
    pub media_type: String,
    pub size: i64,
    /// The manifest this one is *about*, for the OCI referrers API.
    pub subject_digest: Option<String>,
    pub artifact_type: Option<String>,
    pub updated_at: DateTime<FixedOffset>,
    pub created_at: DateTime<FixedOffset>,
}

impl DBDockerManifest {
    #[instrument(skip(database))]
    pub async fn get(
        repository_id: Uuid,
        image_name: &str,
        digest: &str,
        database: &PgPool,
    ) -> Result<Option<Self>, sqlx::Error> {
        SelectQueryBuilder::with_columns(Self::table_name(), Self::columns())
            .filter(DBDockerManifestColumn::RepositoryId.equals(repository_id.value()))
            .filter(DBDockerManifestColumn::ImageName.equals(image_name.value()))
            .filter(DBDockerManifestColumn::Digest.equals(digest.value()))
            .query_as()
            .fetch_optional(database)
            .await
    }

    /// Whether a manifest exists, without reading it back.
    ///
    /// Used to check that every child of an index is already present before the index is accepted.
    #[instrument(skip(database))]
    pub async fn exists(
        repository_id: Uuid,
        image_name: &str,
        digest: &str,
        database: &PgPool,
    ) -> Result<bool, sqlx::Error> {
        let count: i64 = sqlx::query_scalar(
            r#"SELECT COUNT(*) FROM docker_manifests
               WHERE repository_id = $1 AND image_name = $2 AND digest = $3"#,
        )
        .bind(repository_id)
        .bind(image_name)
        .bind(digest)
        .fetch_one(database)
        .await?;
        Ok(count > 0)
    }

    /// Records a manifest, or refreshes what is already recorded for that digest.
    ///
    /// A digest identifies its bytes, so a second push of the same manifest is the same manifest —
    /// re-pushing an image (which every CI run does) must not be a unique-constraint violation.
    #[instrument(skip(database))]
    #[allow(clippy::too_many_arguments)]
    pub async fn upsert(
        repository_id: Uuid,
        image_name: &str,
        digest: &str,
        media_type: &str,
        size: i64,
        subject_digest: Option<&str>,
        artifact_type: Option<&str>,
        database: &PgPool,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"INSERT INTO docker_manifests
                 (repository_id, image_name, digest, media_type, size, subject_digest, artifact_type)
               VALUES ($1, $2, $3, $4, $5, $6, $7)
               ON CONFLICT (repository_id, image_name, digest)
               DO UPDATE SET media_type = EXCLUDED.media_type,
                             size = EXCLUDED.size,
                             subject_digest = EXCLUDED.subject_digest,
                             artifact_type = EXCLUDED.artifact_type,
                             updated_at = CURRENT_TIMESTAMP"#,
        )
        .bind(repository_id)
        .bind(image_name)
        .bind(digest)
        .bind(media_type)
        .bind(size)
        .bind(subject_digest)
        .bind(artifact_type)
        .execute(database)
        .await?;
        Ok(())
    }

    /// Every manifest that declares `digest` as its subject — the OCI referrers list.
    #[instrument(skip(database))]
    pub async fn referrers(
        repository_id: Uuid,
        image_name: &str,
        subject_digest: &str,
        database: &PgPool,
    ) -> Result<Vec<Self>, sqlx::Error> {
        SelectQueryBuilder::with_columns(Self::table_name(), Self::columns())
            .filter(DBDockerManifestColumn::RepositoryId.equals(repository_id.value()))
            .filter(DBDockerManifestColumn::ImageName.equals(image_name.value()))
            .filter(DBDockerManifestColumn::SubjectDigest.equals(subject_digest.value()))
            .order_by(DBDockerManifestColumn::CreatedAt, SQLOrder::Ascending)
            .query_as()
            .fetch_all(database)
            .await
    }

    #[instrument(skip(database))]
    pub async fn delete(
        repository_id: Uuid,
        image_name: &str,
        digest: &str,
        database: &PgPool,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(
            r#"DELETE FROM docker_manifests
               WHERE repository_id = $1 AND image_name = $2 AND digest = $3"#,
        )
        .bind(repository_id)
        .bind(image_name)
        .bind(digest)
        .execute(database)
        .await?;
        Ok(result.rows_affected() > 0)
    }
}

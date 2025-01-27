use crate::builder_error::BuilderError;
use crate::{
    database::prelude::*,
    repository::project::{ReleaseType, VersionData},
};
use derive_builder::Builder;
use sqlx::types::Json;
use uuid::Uuid;

use super::DBProjectVersion;

#[derive(Debug, Clone, PartialEq, Eq, Builder)]
#[builder(build_fn(error = "BuilderError"))]
pub struct NewVersion {
    pub project_id: Uuid,
    /// The version of the project
    pub version: String,
    /// Release type
    pub release_type: ReleaseType,
    /// The path to the release
    pub version_path: String,
    /// The publisher of the version
    #[builder(default)]
    pub publisher: Option<i32>,
    /// The version page. Such as a README
    #[builder(default)]
    pub version_page: Option<String>,
    /// The version data. More data can be added in the future and the data can be repository dependent
    pub extra: VersionData,
}
impl NewVersion {
    pub async fn insert(self, db: &PgPool) -> Result<DBProjectVersion, sqlx::Error> {
        let Self {
            project_id,
            version,
            release_type,
            version_path,
            publisher,
            version_page,
            extra,
        } = self;
        let db_version =  sqlx::query_as(
            r#"
            INSERT INTO project_versions (project_id, version, release_type, version_path, publisher, version_page, extra)
            VALUES ($1, $2, $3, $4, $5, $6, $7) RETURNING *
            "#,
        )
        .bind(project_id)
        .bind(&version)
        .bind(release_type.to_string())
        .bind(version_path)
        .bind(publisher)
        .bind(version_page)
        .bind(Json(extra))
        .fetch_one(db)
        .await?;
        match release_type {
            ReleaseType::Stable => {
                sqlx::query(
                    r#"
                    UPDATE projects
                    SET latest_release = $1, latest_pre_release = $1
                    WHERE id = $2
                    "#,
                )
                .bind(version)
                .bind(project_id)
                .execute(db)
                .await?;
            }
            ReleaseType::Unknown => {
                info!("Unknown release type for version {}", version);
            }
            _ => {
                sqlx::query(
                    r#"
                    UPDATE projects
                    SET latest_pre_release = $1
                    WHERE id = $2
                    "#,
                )
                .bind(version)
                .bind(project_id)
                .execute(db)
                .await?;
            }
        }

        Ok(db_version)
    }
}

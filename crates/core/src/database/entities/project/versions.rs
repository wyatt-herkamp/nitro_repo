use serde::Serialize;
use sqlx::{types::Json, PgPool};
use utoipa::ToSchema;
use uuid::Uuid;
mod new;
use super::ProjectIds;
use crate::{
    database::prelude::*,
    repository::project::{ReleaseType, VersionData},
};
pub use new::*;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, FromRow, ToSchema, Columns)]
pub struct DBProjectVersion {
    pub id: i32,
    /// A reference to the project
    pub project_id: Uuid,
    /// The version of the project
    pub version: String,
    /// Release type
    pub release_type: ReleaseType,
    /// The path to the release
    pub version_path: String,
    /// The publisher of the version
    pub publisher: Option<i32>,
    /// The version page. Such as a README
    pub version_page: Option<String>,
    /// The version data. More data can be added in the future and the data can be repository dependent
    #[schema(value_type = VersionData)]
    pub extra: Json<VersionData>,
    /// When the version was created
    pub updated_at: DateTime<FixedOffset>,
    pub created_at: DateTime<FixedOffset>,
}
impl TableType for DBProjectVersion {
    type Columns = DBProjectVersionColumn;

    fn table_name() -> &'static str
    where
        Self: Sized,
    {
        "project_versions"
    }
}
impl DBProjectVersion {
    #[instrument(skip(database))]
    pub async fn find_by_version_and_project(
        version: &str,
        project_id: Uuid,
        database: &PgPool,
    ) -> Result<Option<Self>, sqlx::Error> {
        let version = sqlx::query_as::<_, Self>(
            r#"SELECT * FROM project_versions WHERE project_id = $1 AND version = $2"#,
        )
        .bind(project_id)
        .bind(version)
        .fetch_optional(database)
        .await?;
        Ok(version)
    }
    #[instrument(skip(database))]
    pub async fn find_by_version_directory(
        directory: &str,
        repository_id: Uuid,
        database: &PgPool,
    ) -> Result<Option<Self>, sqlx::Error> {
        let version = sqlx::query_as::<_, Self>(
            r#"SELECT project_versions.* FROM project_versions FULL JOIN projects ON projects.id = project_versions.project_id AND projects.repository_id = $1 WHERE LOWER(project_versions.version_path) = $2"#,
        )
        .bind(repository_id)
        .bind(directory.to_lowercase())
        .fetch_optional(database)
        .await?;
        Ok(version)
    }
    #[instrument(skip(database))]
    pub async fn find_ids_by_version_dir(
        directory: &str,
        repository_id: Uuid,
        database: &PgPool,
    ) -> Result<Option<ProjectIds>, sqlx::Error> {
        let version = sqlx::query_as::<_, ProjectIds>(
            r#"SELECT project_versions.id as version_id, project_versions.project_id as project_id FROM project_versions FULL JOIN projects ON projects.id = project_versions.project_id AND projects.repository_id = $1 WHERE LOWER(project_versions.version_path) = $2"#,
        )
        .bind(repository_id)
        .bind(directory.to_lowercase())
        .fetch_optional(database)
        .await?;
        Ok(version)
    }
    pub async fn get_all_versions(
        project_id: Uuid,
        database: &PgPool,
    ) -> Result<Vec<Self>, sqlx::Error> {
        let versions =
            sqlx::query_as::<_, Self>(r#"SELECT * FROM project_versions WHERE project_id = $1"#)
                .bind(project_id)
                .fetch_all(database)
                .await?;
        Ok(versions)
    }
}

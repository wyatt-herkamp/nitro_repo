use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use uuid::Uuid;
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct ProjectInfo {
    pub project_id: Uuid,
    pub project_key: String,
    pub project_scope: String,
    pub project_name: String,
    #[sqlx(default)]
    pub project_version: Option<String>,
    #[sqlx(default)]
    pub version_id: Option<i32>,
}
impl ProjectInfo {
    pub async fn query_from_version_id(
        version_id: i32,
        database: &sqlx::PgPool,
    ) -> Result<Option<Self>, sqlx::Error> {
        let project_info = sqlx::query_as::<_, Self>(
            r#"SELECT projects.id as project_id,
                         projects.project_key as project_key,
                         projects.scope as project_scope,
                         projects.name as project_name,
                         project_versions.id as version_id,
                         project_versions.version as project_version
                         FROM project_versions
                            FULL JOIN projects ON projects.id = project_versions.project_id
                        WHERE project_versions.id = $1"#,
        )
        .bind(version_id)
        .fetch_optional(database)
        .await?;
        Ok(project_info)
    }

    pub async fn query_from_project_id(
        project_id: Uuid,
        database: &sqlx::PgPool,
    ) -> Result<Option<Self>, sqlx::Error> {
        let project_info = sqlx::query_as::<_, Self>(
            r#"SELECT
                        projects.id as project_id,
                         projects.project_key as project_key,
                         projects.scope as project_scope,
                         projects.name as project_name
                         FROM projects
                         WHERE projects.id = $1"#,
        )
        .bind(project_id)
        .fetch_optional(database)
        .await?;
        Ok(project_info)
    }
}

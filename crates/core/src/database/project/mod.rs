use derive_builder::Builder;
use serde::Serialize;
use sqlx::{postgres::PgRow, types::Json, FromRow, PgPool};
use tracing::instrument;
use utoipa::ToSchema;
use uuid::Uuid;
mod new;
pub mod utils;
use crate::repository::project::{ReleaseType, VersionData};
pub use new::*;
pub mod update;
use super::DateTime;
/// Implemented on different types of Project query result. Such as ProjectLookupResult
pub trait ProjectDBType: for<'r> FromRow<'r, PgRow> + Unpin + Send + Sync {
    /// What columns to select from the database
    fn columns() -> Vec<&'static str>;
    fn format_columns(prefix: Option<&str>) -> String {
        if let Some(prefix) = prefix {
            Self::columns()
                .iter()
                .map(|column| format!("{}.{}", prefix, column))
                .collect::<Vec<String>>()
                .join(", ")
        } else {
            Self::columns().join(", ")
        }
    }
    async fn find_by_id(id: Uuid, database: &PgPool) -> Result<Option<Self>, sqlx::Error> {
        let columns = Self::format_columns(None);
        let project =
            sqlx::query_as::<_, Self>(&format!("SELECT {} FROM projects WHERE id = $1", columns))
                .bind(id)
                .fetch_optional(database)
                .await?;
        Ok(project)
    }
    async fn find_by_project_key(
        project_key: &str,
        repository: Uuid,
        database: &PgPool,
    ) -> Result<Option<Self>, sqlx::Error> {
        let columns = Self::format_columns(None);
        let project = sqlx::query_as::<_, Self>(&format!(
            "SELECT {} FROM projects WHERE repository_id = $1 AND LOWER(project_key) = $2",
            columns
        ))
        .bind(repository)
        .bind(project_key.to_lowercase())
        .fetch_optional(database)
        .await?;
        Ok(project)
    }
    #[instrument(skip(database), name = "ProjectDBType::find_by_project_directory")]
    async fn find_by_project_directory(
        directory: &str,
        repository: Uuid,
        database: &PgPool,
    ) -> Result<Option<Self>, sqlx::Error> {
        let columns = Self::format_columns(None);
        let project = sqlx::query_as::<_, Self>(&format!(
            "SELECT {} FROM projects WHERE repository_id = $1 AND LOWER(storage_path) = $2",
            columns
        ))
        .bind(repository)
        .bind(directory.to_lowercase())
        .fetch_optional(database)
        .await?;
        Ok(project)
    }
    #[instrument(skip(database), name = "ProjectDBType::find_by_version_directory")]
    async fn get_by_id(id: Uuid, database: &PgPool) -> Result<Option<Self>, sqlx::Error> {
        let columns = Self::format_columns(None);

        let project =
            sqlx::query_as::<_, Self>(&format!("SELECT {} FROM projects WHERE id = $1", columns))
                .bind(id)
                .fetch_optional(database)
                .await?;
        Ok(project)
    }
    /// Finds a Project by the directory of the version
    async fn find_by_version_directory(
        directory: &str,
        repository: Uuid,
        database: &PgPool,
    ) -> Result<Option<Self>, sqlx::Error> {
        let columns = Self::format_columns(Some("P"));
        let project = sqlx::query_as::<_, Self>(
            &format!(
                "SELECT {} FROM projects as P FULL JOIN project_versions as V ON LOWER(V.release_path) = $1 AND V.project_id = P.id WHERE P.repository_id = $2",
                columns
            ),
        )
        .bind(directory.to_lowercase())
        .bind(repository)
        .fetch_optional(database)
        .await?;
        Ok(project)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, FromRow, ToSchema, Builder)]
pub struct DBProject {
    pub id: Uuid,
    /// Maven will use the groupId
    /// Cargo will be None
    /// NPM will use scope if it's set
    pub scope: Option<String>,
    /// Maven will use something like `{groupId}:{artifactId}`
    /// Cargo will use the `name` field
    pub project_key: String,
    /// Name of the project
    ///
    /// Maven will use the artifactId
    /// Cargo will use the `name` field
    /// NPM will use the `name` field
    pub name: String,
    /// Latest stable release
    pub latest_release: Option<String>,
    /// Release is SNAPSHOT in Maven or Alpha, Beta, on any other repository type
    /// This is the latest release or pre-release
    pub latest_pre_release: Option<String>,
    /// A short description of the project
    pub description: Option<String>,
    /// Can be empty
    pub tags: Vec<String>,
    /// The repository it belongs to
    pub repository_id: Uuid,
    /// Storage Path
    pub storage_path: String,
    /// Last time the project was updated. This is updated when a new version is added
    pub updated_at: DateTime,
    /// When the project was created
    pub created_at: DateTime,
}
impl ProjectDBType for DBProject {
    fn columns() -> Vec<&'static str> {
        vec!["*"]
    }
}
/// On the first push. The pusher will be added as a project member with write and manage permissions
#[derive(Debug, Clone, PartialEq, Eq, Serialize, FromRow, ToSchema)]
pub struct DBProjectMember {
    pub id: i32,
    pub project_id: Uuid,
    pub user_id: i32,
    pub can_write: bool,
    pub can_manage: bool,
    pub added: DateTime,
}
impl DBProjectMember {}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, FromRow, ToSchema)]
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
    pub extra: Json<VersionData>,
    /// When the version was created
    pub updated_at: DateTime,
    pub created_at: DateTime,
}
impl DBProjectVersion {
    #[instrument(skip(database), name = "DBProjectVersion::find_by_version_and_project")]
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
    #[instrument(skip(database), name = "DBProjectVersion::find_by_version_directory")]
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

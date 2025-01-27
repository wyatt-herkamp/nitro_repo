use crate::builder_error::BuilderError;
use crate::repository::project::{ReleaseType, VersionData};
use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use sqlx::{types::Json, PgPool};
use tracing::info;
use utoipa::ToSchema;
use uuid::Uuid;

use super::DBProject;
#[derive(Debug, Clone, PartialEq, Eq, Builder)]
#[builder(build_fn(error = "BuilderError"))]

pub struct NewProject {
    #[builder(default)]
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
    #[builder(default)]
    pub latest_release: Option<String>,
    /// Release is SNAPSHOT in Maven or Alpha, Beta, on any other repository type
    /// This is the latest release or pre-release
    #[builder(default)]
    pub latest_pre_release: Option<String>,
    /// A short description of the project
    #[builder(default)]
    pub description: Option<String>,
    /// Can be empty
    #[builder(default)]
    pub tags: Vec<String>,
    /// The repository it belongs to
    pub repository: Uuid,
    /// Storage Path
    pub storage_path: String,
}
impl NewProject {
    pub async fn insert(self, db: &sqlx::PgPool) -> Result<DBProject, sqlx::Error> {
        let Self {
            scope,
            project_key,
            name,
            latest_release,
            latest_pre_release,
            description,
            tags,
            repository,
            storage_path,
        } = self;

        let insert = sqlx::query_as::<_,DBProject>(
            r#"
            INSERT INTO projects (scope, project_key, name, latest_release, latest_pre_release, description, tags, repository_id, storage_path)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9) RETURNING *
            "#
        ).bind(scope)
        .bind(project_key)
        .bind(name)
        .bind(latest_release)
        .bind(latest_pre_release)
        .bind(description)
        .bind(tags)
        .bind(repository)
        .bind(storage_path)
        .fetch_one(db).await?;
        Ok(insert)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
pub struct NewProjectMember {
    pub user_id: i32,
    pub project_id: Uuid,
    pub can_write: bool,
    pub can_manage: bool,
}
impl NewProjectMember {
    pub fn new_owner(user_id: i32, project: Uuid) -> Self {
        Self {
            user_id,
            project_id: project,
            can_write: true,
            can_manage: true,
        }
    }
    pub async fn insert_no_return(self, db: &PgPool) -> Result<(), sqlx::Error> {
        let Self {
            user_id,
            project_id,
            can_write,
            can_manage,
        } = self;
        sqlx::query(
            r#"
            INSERT INTO project_members (user_id, project_id, can_write, can_manage)
            VALUES ($1, $2, $3, $4)
            "#,
        )
        .bind(user_id)
        .bind(project_id)
        .bind(can_write)
        .bind(can_manage)
        .execute(db)
        .await?;
        Ok(())
    }
}

use pg_extended_sqlx_queries::{DynEncodeType, InsertQueryBuilder, QueryTool, TableType};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use tracing::{Span, field::display, instrument};
use utoipa::ToSchema;
use uuid::Uuid;

use super::{DBProject, DBProjectColumn};
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewProject {
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
    /// A short description of the project
    pub description: Option<String>,
    /// The repository it belongs to
    pub repository: Uuid,
    /// Storage Path
    pub storage_path: String,
}
impl NewProject {
    #[instrument(fields(project.id), name = "New Project Insert")]
    pub async fn insert(self, db: &sqlx::PgPool) -> Result<DBProject, sqlx::Error> {
        let Self {
            scope,
            project_key,
            name,
            description,
            repository,
            storage_path,
        } = self;
        let result: DBProject = InsertQueryBuilder::new(DBProject::table_name())
            .insert(DBProjectColumn::Scope, scope.value())
            .insert(DBProjectColumn::Key, project_key.value())
            .insert(DBProjectColumn::Name, name.value())
            .insert(DBProjectColumn::Description, description.value())
            .insert(DBProjectColumn::RepositoryId, repository.value())
            .insert(DBProjectColumn::Path, storage_path.value())
            .return_all()
            .query_as()
            .fetch_one(db)
            .await?;

        Span::current().record("project.id", display(&result.id));

        Ok(result)
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

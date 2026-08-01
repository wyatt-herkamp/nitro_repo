use serde::Serialize;
use utoipa::ToSchema;
use uuid::Uuid;
mod new;
pub use new::*;

use crate::database::prelude::*;

/// On the first push. The pusher will be added as a project member with write and manage permissions
#[derive(Debug, Clone, PartialEq, Eq, Serialize, FromRow, ToSchema, TableType)]
#[table(name = "project_members")]
pub struct DBProjectMember {
    pub id: i32,
    pub project_id: Uuid,
    pub user_id: i32,
    pub can_write: bool,
    pub can_manage: bool,
    pub added: chrono::DateTime<chrono::FixedOffset>,
}
impl DBProjectMember {
    /// Whether a user may push to a project.
    ///
    /// Needed by Maven's `must_be_project_member` push rule, which was defined and never read.
    #[instrument(skip(database))]
    pub async fn can_write(
        project_id: Uuid,
        user_id: i32,
        database: &PgPool,
    ) -> Result<bool, sqlx::Error> {
        let can_write: Option<bool> = sqlx::query_scalar(
            r#"SELECT can_write FROM project_members WHERE project_id = $1 AND user_id = $2"#,
        )
        .bind(project_id)
        .bind(user_id)
        .fetch_optional(database)
        .await?;
        Ok(can_write.unwrap_or(false))
    }
}

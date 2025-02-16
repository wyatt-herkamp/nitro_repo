use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use utoipa::ToSchema;
use uuid::Uuid;

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

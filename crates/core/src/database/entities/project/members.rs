use serde::Serialize;
use utoipa::ToSchema;
use uuid::Uuid;
mod new;
use crate::database::prelude::*;
pub use new::*;

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
impl DBProjectMember {}

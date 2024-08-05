use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, Clone, Default, ToSchema)]
#[serde(default)]
pub struct UserPermissions {
    pub admin: bool,
    pub user_manager: bool,
    pub repository_manager: bool,
    pub default_repository_permissions: RepositoryActions,
    pub repository_permissions: Vec<RepositoryPermission>,
}
impl UserPermissions {
    pub fn admin() -> Self {
        Self {
            admin: true,
            user_manager: true,
            repository_manager: true,
            default_repository_permissions: RepositoryActions {
                can_read: true,
                can_write: true,
                can_yank: true,
                can_edit: true,
            },
            repository_permissions: vec![],
        }
    }
}
pub trait HasPermissions {
    fn get_permissions(&self) -> &UserPermissions;

    fn can_edit_repository(&self, repository: Uuid) -> bool {
        self.get_permissions().admin
            || self.get_permissions().repository_manager
            || self
                .get_permissions()
                .repository_permissions
                .iter()
                .find(|p| p.repository == repository)
                .map(|p| p.actions.can_edit)
                .unwrap_or(false)
    }
    fn can_view_repositories(&self) -> bool {
        self.get_permissions().admin || self.get_permissions().repository_manager
    }
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone, ToSchema)]
pub struct RepositoryPermission {
    pub repository: Uuid,
    pub actions: RepositoryActions,
}
#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone, Default, ToSchema)]
pub struct RepositoryActions {
    pub can_read: bool,
    pub can_write: bool,
    pub can_yank: bool,
    pub can_edit: bool,
}

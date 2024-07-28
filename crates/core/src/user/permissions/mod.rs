use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, Clone, Default)]
#[serde(default)]
pub struct UserPermissions {
    pub disabled: bool,
    pub admin: bool,
    pub user_manager: bool,
    pub repository_manager: bool,
    pub default_repository_permissions: RepositoryActions,
    pub repository_permissions: Vec<RepositoryPermission>,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
pub struct RepositoryPermission {
    pub repository: Uuid,
    pub actions: RepositoryActions,
}
#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone, Default)]
pub struct RepositoryActions {
    pub can_read: bool,
    pub can_write: bool,
    pub can_yank: bool,
}

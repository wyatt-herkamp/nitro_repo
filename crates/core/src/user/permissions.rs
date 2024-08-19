use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
/// User permissions
///
/// Default permissions are allowed to read and write to repositories but nothing else
#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, Clone, ToSchema)]
#[serde(default)]
pub struct UserPermissions {
    pub admin: bool,
    pub user_manager: bool,
    /// Storage Manager will be able to create and delete storage locations
    pub storage_manager: bool,
    /// Repository Manager will be able to create and delete repositories
    /// Also will have full read/write access to all repositories
    pub repository_manager: bool,
    pub default_repository_permissions: RepositoryActions,
    pub repository_permissions: HashMap<Uuid, RepositoryActions>,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct UpdatePermissions {
    pub admin: Option<bool>,
    pub user_manager: Option<bool>,
    pub storage_manager: Option<bool>,
    pub repository_manager: Option<bool>,
    pub default_repository_permissions: Option<RepositoryActions>,
    pub repository_permissions: Option<HashMap<Uuid, RepositoryActions>>,
}
impl UpdatePermissions {
    pub fn apply(self, permissions: &mut UserPermissions) {
        if let Some(admin) = self.admin {
            permissions.admin = admin;
        }
        if let Some(user_manager) = self.user_manager {
            permissions.user_manager = user_manager;
        }
        if let Some(storage_manager) = self.storage_manager {
            permissions.storage_manager = storage_manager;
        }
        if let Some(repository_manager) = self.repository_manager {
            permissions.repository_manager = repository_manager;
        }
        if let Some(default_repository_permissions) = self.default_repository_permissions {
            permissions.default_repository_permissions = default_repository_permissions;
        }
        if let Some(repository_permissions) = self.repository_permissions {
            permissions.repository_permissions = repository_permissions;
        }
    }
}
impl Default for UserPermissions {
    fn default() -> Self {
        Self {
            admin: false,
            storage_manager: false,
            user_manager: false,
            repository_manager: false,
            default_repository_permissions: RepositoryActions {
                can_read: true,
                can_write: true,
                can_edit: false,
            },
            repository_permissions: HashMap::new(),
        }
    }
}
impl UserPermissions {
    pub fn admin() -> Self {
        Self {
            admin: true,
            storage_manager: true,
            user_manager: true,
            repository_manager: true,
            default_repository_permissions: RepositoryActions {
                can_read: true,
                can_write: true,
                can_edit: true,
            },
            repository_permissions: HashMap::new(),
        }
    }
    pub fn delete_repository(&mut self, repository: Uuid) {
        self.repository_permissions.remove(&repository);
    }
}
impl HasPermissions for UserPermissions {
    #[inline(always)]
    fn get_permissions(&self) -> Option<&UserPermissions> {
        Some(self)
    }
}
impl<HS: HasPermissions> HasPermissions for Option<HS> {
    fn get_permissions(&self) -> Option<&UserPermissions> {
        self.as_ref().and_then(HasPermissions::get_permissions)
    }
}
pub trait HasPermissions {
    /// Get the permissions of the user. If the user or not logged in, return None
    fn get_permissions(&self) -> Option<&UserPermissions>;
    /// Is the user an admin
    #[inline(always)]
    fn is_admin_or_user_manager(&self) -> bool {
        self.get_permissions()
            .map(|p| p.admin || p.user_manager)
            .unwrap_or(false)
    }
    /// Is the user an admin or repository manager
    #[inline(always)]
    fn is_admin_or_repository_manager(&self) -> bool {
        self.get_permissions()
            .map(|p| p.admin || p.repository_manager)
            .unwrap_or(false)
    }
    #[inline(always)]
    fn is_admin_or_storage_manager(&self) -> bool {
        self.get_permissions()
            .map(|p| p.admin || p.storage_manager)
            .unwrap_or(false)
    }
    /// Can a user edit the repository settings
    ///
    /// True if the user is an admin, repository manager, or has the correct permissions
    fn can_edit_repository(&self, repository: Uuid) -> bool {
        if self.is_admin_or_repository_manager() {
            return true;
        }
        let Some(permissions) = self.get_permissions() else {
            return false;
        };
        permissions
            .repository_permissions
            .get(&repository)
            .map(|p| p.can_edit)
            .unwrap_or(permissions.default_repository_permissions.can_edit)
    }
    /// Can a user write to the repository
    ///
    /// True if the user is an admin, repository manager, or has the correct permissions
    fn can_write_to_repository(&self, repository: Uuid) -> bool {
        if self.is_admin_or_repository_manager() {
            return true;
        }
        let Some(permissions) = self.get_permissions() else {
            return false;
        };
        permissions
            .repository_permissions
            .get(&repository)
            .map(|p| p.can_edit)
            .unwrap_or(permissions.default_repository_permissions.can_write)
    }
    /// Can a user read from the repository
    ///
    /// True if the user is an admin, repository manager, or has the correct permissions
    fn can_read_repository(&self, repository: Uuid) -> bool {
        if self.is_admin_or_repository_manager() {
            return true;
        }
        let Some(permissions) = self.get_permissions() else {
            return false;
        };
        permissions
            .repository_permissions
            .get(&repository)
            .map(|p| p.can_edit)
            .unwrap_or(permissions.default_repository_permissions.can_read)
    }
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone, Default, ToSchema)]
pub struct RepositoryActions {
    /// Can the user read/pull from this repository
    /// This is ignored if the repository is set to public
    pub can_read: bool,
    /// Can the user write/push to this repository
    pub can_write: bool,
    /// Can the user edit this repositories settings
    pub can_edit: bool,
}

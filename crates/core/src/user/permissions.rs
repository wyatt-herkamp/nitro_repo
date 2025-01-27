use std::fmt::Debug;

use ahash::HashMap;
use serde::{Deserialize, Serialize};
use sqlx::{
    prelude::{FromRow, Type},
    Execute, PgPool, QueryBuilder,
};
use tracing::{debug, info, instrument, trace, warn};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::database::entities::user::{
    auth_token::AuthToken,
    permissions::{NewUserRepositoryPermissions, UserRepositoryPermissions},
};

use super::scopes::NRScope;
/// User permissions
///
/// Default permissions are allowed to read and write to repositories but nothing else
#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, Clone, ToSchema, FromRow)]
pub struct UserPermissions {
    pub id: i32,
    pub admin: bool,
    pub user_manager: bool,
    /// Repository Manager will be able to create and delete repositories
    /// Also will have full read/write access to all repositories
    pub system_manager: bool,
    pub default_repository_actions: Vec<RepositoryActions>,
}

impl HasPermissions for UserPermissions {
    #[inline(always)]
    fn get_permissions(&self) -> Option<UserPermissions> {
        Some(self.clone())
    }

    fn user_id(&self) -> Option<i32> {
        Some(self.id)
    }
}
impl<HS: HasPermissions> HasPermissions for Option<HS> {
    fn get_permissions(&self) -> Option<UserPermissions> {
        self.as_ref().and_then(HasPermissions::get_permissions)
    }

    fn user_id(&self) -> Option<i32> {
        self.as_ref().and_then(HasPermissions::user_id)
    }
}
pub trait HasPermissions {
    fn user_id(&self) -> Option<i32>;
    /// Get the permissions of the user. If the user or not logged in, return None
    fn get_permissions(&self) -> Option<UserPermissions>;
    /// Is the user an admin
    #[inline(always)]
    fn is_admin_or_user_manager(&self) -> bool {
        self.get_permissions()
            .map(|p| p.admin || p.user_manager)
            .unwrap_or(false)
    }
    /// Is the user an admin or repository manager
    #[inline(always)]
    fn is_admin_or_system_manager(&self) -> bool {
        self.get_permissions()
            .map(|p| p.admin || p.system_manager)
            .unwrap_or(false)
    }

    async fn has_action(
        &self,
        action: RepositoryActions,
        repository: Uuid,
        db: &PgPool,
    ) -> Result<bool, sqlx::Error> {
        if self.is_admin_or_system_manager() {
            return Ok(true);
        }
        let Some(user_id) = self.user_id() else {
            return Ok(false);
        };
        UserRepositoryPermissions::has_repository_action(user_id, repository, action, db).await
    }
}
/// Checks if the Auth Token has the scope for the action and that the user has permission for it.
///
/// Yes this is a big function name
#[instrument]
pub async fn does_user_and_token_have_repository_action<T: HasPermissions + Debug>(
    user: &T,
    token: &AuthToken,
    action: RepositoryActions,
    repository: Uuid,
    database: &PgPool,
) -> sqlx::Result<bool> {
    if !user.has_action(action, repository, database).await? {
        debug!("User does not have permission for action. Not checking Auth Token");
        return Ok(false);
    }
    trace!("User Has Permission. Checking Auth Token has necessary scope");
    token
        .has_repository_action(repository, action, database)
        .await
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type, ToSchema)]
#[sqlx(type_name = "TEXT")]
pub enum RepositoryActions {
    Read,
    Write,
    Edit,
}
impl RepositoryActions {
    pub fn all() -> Vec<Self> {
        vec![Self::Read, Self::Write, Self::Edit]
    }
}
impl From<RepositoryActions> for NRScope {
    fn from(action: RepositoryActions) -> Self {
        match action {
            RepositoryActions::Read => NRScope::ReadRepository,
            RepositoryActions::Write => NRScope::WriteRepository,
            RepositoryActions::Edit => NRScope::EditRepository,
        }
    }
}

#[derive(Debug, Deserialize, Clone, ToSchema)]
pub struct UpdatePermissions {
    pub admin: Option<bool>,
    pub user_manager: Option<bool>,
    pub system_manager: Option<bool>,
    pub default_repository_actions: Option<Vec<RepositoryActions>>,
    #[serde(default)]
    pub repository_permissions: HashMap<Uuid, Vec<RepositoryActions>>,
}
impl UpdatePermissions {
    pub fn has_regular_change(&self) -> bool {
        self.admin.is_some()
            || self.user_manager.is_some()
            || self.system_manager.is_some()
            || self.default_repository_actions.is_some()
    }
    #[instrument(name = "UpdatePermissions::update_regular")]
    async fn update_regular(&self, user_id: i32, db: &PgPool) -> Result<(), sqlx::Error> {
        let mut query = QueryBuilder::new("UPDATE users SET ");
        let mut separated = query.separated(", ");
        if let Some(admin) = self.admin {
            separated.push("admin = ");
            separated.push_bind_unseparated(admin);
        }
        if let Some(user_manager) = self.user_manager {
            separated.push("user_manager = ");
            separated.push_bind_unseparated(user_manager);
        }
        if let Some(system_manager) = self.system_manager {
            separated.push("system_manager = ");
            separated.push_bind_unseparated(system_manager);
        }
        if let Some(default_repository_actions) = &self.default_repository_actions {
            separated.push("default_repository_actions = ");
            separated.push_bind_unseparated(default_repository_actions);
        }
        query.push(" WHERE id = ");
        query.push_bind(user_id);
        let query = query.build();
        info!("Updating permissions for user {} {}", user_id, query.sql());
        let result = query.execute(db).await?;
        if result.rows_affected() == 0 {
            warn!(
                "No rows affected when updating permissions for user {}",
                user_id
            );
        }
        Ok(())
    }
    #[instrument(name = "UpdatePermissions::update_permissions")]
    pub async fn update_permissions(self, user_id: i32, db: &PgPool) -> Result<(), sqlx::Error> {
        if self.has_regular_change() {
            self.update_regular(user_id, db).await?;
        } else {
            info!("No regular permissions to update");
        }
        if self.repository_permissions.is_empty() {
            info!("No repository permissions to update");
            return Ok(());
        }
        let span = tracing::span!(
            tracing::Level::DEBUG,
            "UpdatePermissions::update_permissions::repository_permissions"
        );
        let _guard = span.enter();
        for (repository, actions) in self.repository_permissions {
            if actions.is_empty() {
                debug!(
                    "Removing entry for repository {} for user {}. Because actions is empty",
                    repository, user_id
                );
                UserRepositoryPermissions::delete(user_id, repository, db).await?;
                continue;
            }

            let permissions = NewUserRepositoryPermissions {
                user_id,
                repository_id: repository,
                actions,
            };
            permissions.insert(db).await?;
        }
        return Ok(());
    }
}

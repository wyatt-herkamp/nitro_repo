use std::{collections::HashMap, fmt::Debug};

use serde::{Deserialize, Serialize};
use sqlx::{
    prelude::{FromRow, Type},
    PgPool,
};
use tracing::{debug, instrument, trace};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::database::{user::auth_token::AuthToken, DateTime};

use super::scopes::Scopes;
/// User permissions
///
/// Default permissions are allowed to read and write to repositories but nothing else
#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, Clone, ToSchema, FromRow)]
pub struct UserPermissions {
    pub id: i32,
    pub admin: bool,
    pub user_manager: bool,
    /// Storage Manager will be able to create and delete storage locations
    pub storage_manager: bool,
    /// Repository Manager will be able to create and delete repositories
    /// Also will have full read/write access to all repositories
    pub repository_manager: bool,
    pub default_repository_actions: Vec<RepositoryActionOptions>,
}
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, ToSchema, FromRow)]

pub struct UserRepositoryPermissions {
    pub id: i32,
    pub user_id: i32,
    pub repository_id: Uuid,
    pub actions: Vec<RepositoryActionOptions>,
    pub updated_at: DateTime,
    pub created_at: DateTime,
}
impl UserRepositoryPermissions {
    pub async fn has_repository_action(
        user_id: i32,
        repository: Uuid,
        action: RepositoryActionOptions,
        database: &PgPool,
    ) -> sqlx::Result<bool> {
        let Some(actions) = sqlx::query_scalar::<_, Vec<RepositoryActionOptions>>(
            r#"SELECT * FROM user_repository_permissions WHERE user_id = $1 AND repository_id = $2 "#,
        )
        .bind(user_id)
        .bind(repository)
        .fetch_optional(database)
        .await? else{
            return Ok(false);
        };
        Ok(actions.contains(&action))
    }
}
#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct UpdatePermissions {
    pub admin: Option<bool>,
    pub user_manager: Option<bool>,
    pub storage_manager: Option<bool>,
    pub repository_manager: Option<bool>,
    pub default_repository_actions: Option<RepositoryActions>,
    pub repository_permissions: Option<HashMap<Uuid, RepositoryActions>>,
}

impl UpdatePermissions {
    pub fn apply(self, permissions: &mut UserPermissions) {
        todo!()
    }
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
    async fn has_action(
        &self,
        action: RepositoryActionOptions,
        repository: Uuid,
        db: &PgPool,
    ) -> Result<bool, sqlx::Error> {
        if self.is_admin_or_repository_manager() {
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
    action: RepositoryActionOptions,
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
impl RepositoryActions {
    pub fn has_action(&self, action: RepositoryActionOptions) -> bool {
        match action {
            RepositoryActionOptions::Read => self.can_read,
            RepositoryActionOptions::Write => self.can_write,
            RepositoryActionOptions::Edit => self.can_edit,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[sqlx(type_name = "TEXT")]
pub enum RepositoryActionOptions {
    Read,
    Write,
    Edit,
}
impl RepositoryActionOptions {
    pub fn all() -> Vec<Self> {
        vec![Self::Read, Self::Write, Self::Edit]
    }
}
impl From<RepositoryActionOptions> for Scopes {
    fn from(action: RepositoryActionOptions) -> Self {
        match action {
            RepositoryActionOptions::Read => Scopes::ReadRepository,
            RepositoryActionOptions::Write => Scopes::WriteRepository,
            RepositoryActionOptions::Edit => Scopes::EditRepository,
        }
    }
}

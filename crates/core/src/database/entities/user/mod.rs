use pg_extended_sqlx_queries::prelude::*;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool, postgres::PgRow, types::Json};
use utoipa::ToSchema;

use crate::user::{
    Email, Username,
    permissions::{HasPermissions, RepositoryActions, UserPermissions},
};

pub mod auth_token;
pub mod password_reset;
pub mod permissions;
pub mod user_utils;
/// Implements on types that references a user in the database.
///
/// Such as APIKeys and PasswordReset tokens.
pub trait ReferencesUser {
    fn user_id(&self) -> i32;

    async fn get_owned_by_user(user_id: i32, database: &PgPool) -> Result<Vec<Self>, sqlx::Error>
    where
        Self: Sized;
}
pub trait UserType:
    for<'r> FromRow<'r, PgRow> + Unpin + Send + Sync + TableQuery<Table = User>
{
    fn get_id(&self) -> i32;

    async fn get_by_id(id: i32, database: &PgPool) -> Result<Option<Self>, sqlx::Error>
    where
        Self: Sized,
    {
        let user = SelectQueryBuilder::with_columns(User::table_name(), Self::columns())
            .filter(UserColumn::Id.equals(id.value()))
            .query_as()
            .fetch_optional(database)
            .await?;
        Ok(user)
    }
    async fn get_all(database: &PgPool) -> Result<Vec<Self>, sqlx::Error> {
        let users = SelectQueryBuilder::with_columns(User::table_name(), Self::columns())
            .query_as()
            .fetch_all(database)
            .await?;
        Ok(users)
    }

    async fn get_by_reference(
        reference: &impl ReferencesUser,
        database: &PgPool,
    ) -> Result<Option<Self>, sqlx::Error>
    where
        Self: Sized,
    {
        Self::get_by_id(reference.user_id(), database).await
    }
    async fn get_by_email(email: &str, database: &PgPool) -> Result<Option<Self>, sqlx::Error> {
        let user = SelectQueryBuilder::with_columns(User::table_name(), Self::columns())
            .filter(UserColumn::Email.equals(email.value()))
            .query_as()
            .fetch_optional(database)
            .await?;
        Ok(user)
    }
    async fn get_by_username_or_email(
        username: impl AsRef<str>,
        database: &PgPool,
    ) -> Result<Option<Self>, sqlx::Error>
    where
        Self: Sized,
    {
        let user = SelectQueryBuilder::with_columns(User::table_name(), Self::columns())
            .filter(
                UserColumn::Email
                    .equals(username.as_ref().value())
                    .or(UserColumn::Username.equals(username.as_ref().value())),
            )
            .query_as()
            .fetch_optional(database)
            .await?;
        Ok(user)
    }

    async fn update_permissions(
        &self,
        permissions: UserPermissions,
        database: &PgPool,
    ) -> Result<(), sqlx::Error>
    where
        Self: Sized,
    {
        sqlx::query("UPDATE users SET permissions = $1 WHERE id = $2")
            .bind(Json(permissions))
            .bind(self.get_id())
            .execute(database)
            .await?;
        Ok(())
    }
    async fn update_password(
        &self,
        password: Option<String>,
        database: &PgPool,
    ) -> Result<(), sqlx::Error>
    where
        Self: Sized,
    {
        sqlx::query("UPDATE users SET password = $1, password_last_changed = NOW(), require_password_change = false WHERE id = $2")
            .bind(password)
            .bind(self.get_id())
            .execute(database)
            .await?;
        Ok(())
    }

    async fn update_email_address(
        &self,
        email: impl AsRef<str>,
        database: &PgPool,
    ) -> Result<(), sqlx::Error>
    where
        Self: Sized,
    {
        sqlx::query("UPDATE users SET email = $1 WHERE id = $2")
            .bind(email.as_ref())
            .bind(self.get_id())
            .execute(database)
            .await?;
        Ok(())
    }

    async fn update_username(
        &self,
        username: impl AsRef<str>,
        database: &PgPool,
    ) -> Result<(), sqlx::Error>
    where
        Self: Sized,
    {
        sqlx::query("UPDATE users SET username = $1 WHERE id = $2")
            .bind(username.as_ref())
            .bind(self.get_id())
            .execute(database)
            .await?;
        Ok(())
    }

    async fn update_name(&self, name: impl AsRef<str>, database: &PgPool) -> Result<(), sqlx::Error>
    where
        Self: Sized,
    {
        sqlx::query("UPDATE users SET name = $1 WHERE id = $2")
            .bind(name.as_ref())
            .bind(self.get_id())
            .execute(database)
            .await?;
        Ok(())
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, FromRow, TableType)]
#[table(name = "users")]
pub struct User {
    pub id: i32,
    pub name: String,
    pub username: Username,
    pub email: Email,
    pub active: bool,
    #[serde(skip_serializing)]
    pub password: Option<String>,
    pub password_last_changed: Option<chrono::DateTime<chrono::FixedOffset>>,
    pub require_password_change: bool,
    pub admin: bool,
    pub user_manager: bool,
    /// Repository Manager will be able to create and delete repositories
    /// Also will have full read/write access to all repositories
    pub system_manager: bool,
    pub default_repository_actions: Vec<RepositoryActions>,
    pub updated_at: chrono::DateTime<chrono::FixedOffset>,
    pub created_at: chrono::DateTime<chrono::FixedOffset>,
}
impl User {
    pub async fn get_password_by_id(
        id: i32,
        database: &PgPool,
    ) -> Result<Option<String>, sqlx::Error> {
        let user_password: Option<String> =
            sqlx::query_scalar("SELECT password FROM users WHERE id = $1")
                .bind(id)
                .fetch_optional(database)
                .await?;
        Ok(user_password)
    }
}

impl UserType for User {
    fn get_id(&self) -> i32 {
        self.id
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, FromRow, ToSchema)]
pub struct UserSafeData {
    pub id: i32,
    pub name: String,
    pub username: Username,
    pub email: Email,
    pub require_password_change: bool,
    pub active: bool,
    pub admin: bool,
    pub user_manager: bool,
    /// Repository Manager will be able to create and delete repositories
    /// Also will have full read/write access to all repositories
    pub system_manager: bool,
    pub default_repository_actions: Vec<RepositoryActions>,
    pub updated_at: chrono::DateTime<chrono::FixedOffset>,
    pub created_at: chrono::DateTime<chrono::FixedOffset>,
}
impl TableQuery for UserSafeData {
    type Table = User;

    fn columns() -> Vec<<Self::Table as TableType>::Columns>
    where
        Self: Sized,
    {
        vec![
            UserColumn::Id,
            UserColumn::Name,
            UserColumn::Username,
            UserColumn::Email,
            UserColumn::RequirePasswordChange,
            UserColumn::Active,
            UserColumn::Admin,
            UserColumn::UserManager,
            UserColumn::SystemManager,
            UserColumn::DefaultRepositoryActions,
            UserColumn::UpdatedAt,
            UserColumn::CreatedAt,
        ]
    }
}
impl UserType for UserSafeData {
    fn get_id(&self) -> i32 {
        self.id
    }
}

impl HasPermissions for UserSafeData {
    fn user_id(&self) -> Option<i32> {
        Some(self.id)
    }

    fn get_permissions(&self) -> Option<UserPermissions> {
        Some(UserPermissions {
            id: self.id,
            admin: self.admin,
            user_manager: self.user_manager,
            system_manager: self.system_manager,
            default_repository_actions: self.default_repository_actions.clone(),
        })
    }
    fn is_admin_or_system_manager(&self) -> bool {
        self.admin || self.system_manager
    }
    fn is_admin_or_user_manager(&self) -> bool {
        self.admin || self.user_manager
    }
}
impl From<User> for UserSafeData {
    fn from(user: User) -> Self {
        UserSafeData {
            id: user.id,
            name: user.name,
            username: user.username,
            email: user.email,
            require_password_change: user.require_password_change,
            active: user.active,
            updated_at: user.updated_at,
            created_at: user.created_at,
            admin: user.admin,
            user_manager: user.user_manager,
            system_manager: user.system_manager,
            default_repository_actions: user.default_repository_actions,
        }
    }
}

#[cfg(test)]
mod tests {
    /// Just incase a bug get's introduced from serde where the password is serialized. We want to error out.
    #[test]
    pub fn assert_no_serialize_password() {
        let user = super::User {
            password: Some("password".to_owned()),
            id: Default::default(),
            name: Default::default(),
            username: "username".parse().unwrap(),
            email: "email@email.com".parse().unwrap(),
            active: Default::default(),
            password_last_changed: Default::default(),
            require_password_change: Default::default(),
            admin: Default::default(),
            user_manager: Default::default(),
            system_manager: Default::default(),
            default_repository_actions: Default::default(),
            updated_at: Default::default(),
            created_at: Default::default(),
        };
        let json = serde_json::to_value(&user).unwrap();

        assert!(
            json.get("password").is_none(),
            "Password should not be serialized"
        );
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct NewUserRequest {
    pub name: String,
    pub username: Username,
    pub email: Email,
    pub password: Option<String>,
}
impl NewUserRequest {
    pub async fn insert(self, database: &PgPool) -> sqlx::Result<User> {
        let Self {
            name,
            username,
            email,
            password,
        } = self;
        let user = sqlx::query_as(
            r#"INSERT INTO users (name, username, email, password) VALUES ($1, $2, $3, $4) RETURNING *"#,
        )
        .bind(name)
        .bind(username)
        .bind(email)
        .bind(password)
        .fetch_one(database).await?;
        Ok(user)
    }
    pub async fn insert_admin(self, database: &PgPool) -> sqlx::Result<User> {
        let Self {
            name,
            username,
            email,
            password,
        } = self;
        let user = sqlx::query_as(
            r#"INSERT INTO users (name, username, email, password, admin, user_manager, system_manager) VALUES ($1, $2, $3, $4, true, true, true) RETURNING *"#,
        )
        .bind(name)
        .bind(username)
        .bind(email)
        .bind(password)
        .fetch_one(database).await?;
        Ok(user)
    }
}
/// Change Password request. That does not check the old password.
/// Used for password reset and admin password changes.
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ChangePasswordNoCheck {
    pub password: String,
}
/// Change Password request. That checks the old password.
/// Used for changing the password when the user is logged in.
#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct ChangePasswordWithCheck {
    pub old_password: String,
    pub new_password: String,
}

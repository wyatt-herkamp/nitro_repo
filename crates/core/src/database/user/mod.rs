use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgRow, types::Json, FromRow, PgPool};
use tracing::instrument;
use utoipa::ToSchema;

use crate::user::{
    permissions::{HasPermissions, UserPermissions},
    Email, Username,
};

use super::DateTime;
pub mod auth_token;
pub mod password_reset;
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
pub trait UserType: for<'r> FromRow<'r, PgRow> + Unpin + Send + Sync {
    fn get_id(&self) -> i32;

    fn columns() -> Vec<&'static str>;
    fn format_columns(prefix: Option<&str>) -> String {
        if let Some(prefix) = prefix {
            Self::columns()
                .iter()
                .map(|column| format!("{}.{}", prefix, column))
                .collect::<Vec<String>>()
                .join(", ")
        } else {
            Self::columns().join(", ")
        }
    }
    async fn get_by_id(id: i32, database: &PgPool) -> Result<Option<Self>, sqlx::Error>
    where
        Self: Sized,
    {
        let columns = Self::format_columns(None);
        let user = sqlx::query_as::<_, Self>(&format!("SELECT {columns} FROM users WHERE id = $1"))
            .bind(id)
            .fetch_optional(database)
            .await?;
        Ok(user)
    }
    async fn get_all(database: &PgPool) -> Result<Vec<Self>, sqlx::Error> {
        let columns = Self::format_columns(None);

        let users = sqlx::query_as::<_, Self>(&format!("SELECT {columns} FROM users"))
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
        let columns: String = Self::format_columns(None);

        let user =
            sqlx::query_as::<_, Self>(&format!("SELECT {columns} FROM users WHERE email = $1"))
                .bind(email)
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
        let columns: String = Self::format_columns(None);

        let user = sqlx::query_as::<_, Self>(&format!(
            "SELECT {columns} FROM users WHERE username = $1 OR email = $1"
        ))
        .bind(username.as_ref())
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
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, FromRow)]
pub struct UserModel {
    pub id: i32,
    pub name: String,
    pub username: Username,
    pub email: Email,
    pub active: bool,
    #[serde(skip_serializing)]
    pub password: Option<String>,
    pub password_last_changed: Option<DateTime>,
    pub require_password_change: bool,

    pub permissions: Json<UserPermissions>,
    pub updated_at: DateTime,
    pub created_at: DateTime,
}
impl UserModel {
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
impl Default for UserModel {
    fn default() -> Self {
        Self {
            id: 0,
            name: Default::default(),
            username: Default::default(),
            email: Default::default(),
            require_password_change: true,
            active: true,
            password_last_changed: None,
            password: Default::default(),
            permissions: Default::default(),
            updated_at: Default::default(),
            created_at: Default::default(),
        }
    }
}
impl UserType for UserModel {
    fn get_id(&self) -> i32 {
        self.id
    }

    fn columns() -> Vec<&'static str> {
        vec!["*"]
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
    #[schema(value_type= UserPermissions)]
    pub permissions: Json<UserPermissions>,
    pub updated_at: DateTime,
    pub created_at: DateTime,
}
impl UserType for UserSafeData {
    fn columns() -> Vec<&'static str> {
        vec![
            "id",
            "name",
            "username",
            "email",
            "require_password_change",
            "active",
            "permissions",
            "updated_at",
            "created_at",
        ]
    }
    fn get_id(&self) -> i32 {
        self.id
    }
}

impl HasPermissions for UserSafeData {
    fn get_permissions(&self) -> Option<&UserPermissions> {
        Some(self.permissions.as_ref())
    }
}
impl From<UserModel> for UserSafeData {
    fn from(user: UserModel) -> Self {
        UserSafeData {
            id: user.id,
            name: user.name,
            username: user.username,
            email: user.email,
            require_password_change: user.require_password_change,
            active: user.active,
            permissions: user.permissions,
            updated_at: user.updated_at,
            created_at: user.created_at,
        }
    }
}

#[cfg(test)]
mod tests {
    /// Just incase a bug get's introduced from serde where the password is serialized. We want to error out.
    #[test]
    pub fn assert_no_serialize_password() {
        let user = super::UserModel {
            password: Some("password".to_owned()),
            ..Default::default()
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
    pub async fn insert(
        self,
        permissions: UserPermissions,
        database: &PgPool,
    ) -> sqlx::Result<UserModel> {
        let Self {
            name,
            username,
            email,
            password,
        } = self;
        let user = sqlx::query_as(
            r#"INSERT INTO users (name, username, email, password, permissions) VALUES ($1, $2, $3, $4, $5) RETURNING *"#,
        )
        .bind(name)
        .bind(username)
        .bind(email)
        .bind(password)
        .bind(Json(permissions))
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

use serde::{Deserialize, Serialize};
use sqlx::{types::Json, FromRow, PgPool};
use utoipa::ToSchema;

use crate::user::permissions::UserPermissions;

use super::DateTime;
pub mod auth_token;
pub mod password_reset;
/// Implements on types that references a user in the database.
///
/// Such as APIKeys and PasswordReset tokens.
pub trait ReferencesUser {
    fn user_id(&self) -> i32;

    async fn get_owned_by_user(user_id: i32, database: &PgPool) -> Result<Vec<Self>, sqlx::Error>
    where
        Self: Sized;
}
pub trait UserType {
    async fn get_by_id(id: i32, database: &PgPool) -> Result<Option<Self>, sqlx::Error>
    where
        Self: Sized;

    async fn get_by_reference(
        reference: &impl ReferencesUser,
        database: &PgPool,
    ) -> Result<Option<Self>, sqlx::Error>
    where
        Self: Sized,
    {
        Self::get_by_id(reference.user_id(), database).await
    }
    async fn get_by_username_or_email(
        username: impl AsRef<str>,
        database: &PgPool,
    ) -> Result<Option<Self>, sqlx::Error>
    where
        Self: Sized;
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, FromRow)]
pub struct UserModel {
    pub id: i32,
    pub name: String,
    pub username: String,
    pub email: String,
    pub active: bool,
    #[serde(skip_serializing)]
    pub password: Option<String>,
    pub password_last_changed: Option<DateTime>,
    pub require_password_change: bool,

    pub permissions: Json<UserPermissions>,
    pub updated_at: DateTime,
    pub created_at: DateTime,
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
    async fn get_by_id(id: i32, database: &PgPool) -> Result<Option<Self>, sqlx::Error> {
        let user = sqlx::query_as::<_, UserModel>("SELECT * FROM users WHERE id = $1")
            .bind(id)
            .fetch_optional(database)
            .await?;
        Ok(user)
    }
    async fn get_by_username_or_email(
        username: impl AsRef<str>,
        database: &PgPool,
    ) -> Result<Option<Self>, sqlx::Error> {
        let user =
            sqlx::query_as::<_, UserModel>("SELECT * FROM users WHERE username = $1 OR email = $1")
                .bind(username.as_ref())
                .fetch_optional(database)
                .await?;
        Ok(user)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, FromRow, ToSchema)]
pub struct UserSafeData {
    pub id: i32,
    pub name: String,
    pub username: String,
    pub email: String,
    pub require_password_change: bool,
    pub active: bool,
    #[schema(value_type= UserPermissions)]
    pub permissions: Json<UserPermissions>,
    pub updated_at: DateTime,
    pub created_at: DateTime,
}
impl UserType for UserSafeData {
    async fn get_by_id(id: i32, database: &PgPool) -> Result<Option<Self>, sqlx::Error> {
        let user = sqlx::query_as::<_, UserSafeData>("SELECT * FROM users WHERE id = $1")
            .bind(id)
            .fetch_optional(database)
            .await?;
        Ok(user)
    }
    async fn get_by_username_or_email(
        username: impl AsRef<str>,
        database: &PgPool,
    ) -> Result<Option<Self>, sqlx::Error> {
        let user = sqlx::query_as::<_, UserSafeData>(
            "SELECT * FROM users WHERE username = $1 OR email = $1",
        )
        .bind(username.as_ref())
        .fetch_optional(database)
        .await?;
        Ok(user)
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
pub async fn does_user_exist(database: &PgPool) -> Result<bool, sqlx::Error> {
    let user = sqlx::query("SELECT id FROM users WHERE active = true LIMIT 1")
        .fetch_optional(database)
        .await?;
    Ok(user.is_some())
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
    pub username: String,
    pub email: String,
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

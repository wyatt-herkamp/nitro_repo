use chrono::Local;
use serde::{Deserialize, Serialize};
use sqlx::{
    prelude::{FromRow, Type},
    types::Json,
    PgPool,
};

use crate::database::DateTime;
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RequestDetails {
    pub ip_address: String,
    pub user_agent: String,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[sqlx(type_name = "TEXT")]
pub enum PasswordResetState {
    Requested,
    Used,
    Invalidated,
    Expired,
}
/// Table Name: `user_password_reset_tokens`
#[derive(Debug, Clone, PartialEq, Eq, FromRow, Serialize)]
pub struct UserPasswordReset {
    pub id: i32,
    pub user_id: i32,
    pub request_details: Json<RequestDetails>,
    pub state: PasswordResetState,
    pub token: String,
    pub expires_at: DateTime,
    pub used_at: DateTime,
    pub created_at: DateTime,
}
impl UserPasswordReset {
    /// Generates a new password reset token for the user
    pub async fn create(
        user_id: i32,
        request_details: RequestDetails,
        database: &PgPool,
    ) -> Result<Self, sqlx::Error> {
        let token = Self::generate_token(database).await?;
        let expires_at = Local::now().fixed_offset() + chrono::Duration::days(1);
        let row = sqlx::query_as::<_, Self>(r#"INSERT INTO user_password_reset_tokens (user_id,request_details, token, expires_at) VALUES ($1, $2, $3, $4) RETURNING *"#,)
            .bind(user_id)
            .bind(Json(request_details))
            .bind(token)
            .bind(expires_at)
            .fetch_one(database)
        .await?;

        Ok(row)
    }
    /// Generate a new password reset token
    async fn generate_token(database: &PgPool) -> Result<String, sqlx::Error> {
        let token = loop {
            let token = Self::generate_token_value();
            if !Self::does_token_exist(&token, database).await? {
                break token;
            }
        };
        Ok(token)
    }
    pub async fn does_token_exist(token: &str, database: &PgPool) -> Result<bool, sqlx::Error> {
        let count: i64 = sqlx::query_scalar(
            r#"SELECT COUNT(id) FROM user_password_reset_tokens  WHERE token = $1"#,
        )
        .bind(token)
        .fetch_one(database)
        .await?;
        Ok(count > 0)
    }
    pub async fn does_token_exist_and_valid(
        token: &str,
        database: &PgPool,
    ) -> Result<bool, sqlx::Error> {
        let count: i64 = sqlx::query_scalar(
            r#"SELECT COUNT(id) FROM user_password_reset_tokens WHERE token = $1 AND state = 'Requested' AND expires_at > NOW()"#,
        )
        .bind(token)
        .fetch_one(database)
        .await?;
        Ok(count > 0)
    }
    pub async fn get_if_valid(token: &str, database: &PgPool) -> Result<Option<Self>, sqlx::Error> {
        let row = sqlx::query_as::<_, Self>(
            r#"SELECT * FROM user_password_reset_tokens WHERE token = $1 AND state = 'Requested' AND expires_at > NOW()"#,
        )
        .bind(token)
        .fetch_optional(database)
        .await?;
        Ok(row)
    }
    fn generate_token_value() -> String {
        use rand::distributions::Alphanumeric;
        use rand::{thread_rng, Rng};

        thread_rng()
            .sample_iter(&Alphanumeric)
            .take(32)
            .map(char::from)
            .collect()
    }
    pub async fn set_used(&self, database: &PgPool) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"UPDATE user_password_reset_tokens SET state = 'Used', used_at = NOW() WHERE id = $1"#,
        )
        .bind(self.id)
        .execute(database)
        .await?;
        Ok(())
    }
}

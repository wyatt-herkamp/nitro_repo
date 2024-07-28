use serde::{Deserialize, Serialize};
use sqlx::{prelude::FromRow, PgPool};

use crate::database::DateTime;

use super::ReferencesUser;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, FromRow)]
pub struct AuthToken {
    pub id: i32,
    pub user_id: i32,
    pub token: String,
    pub active: bool,
    pub expires_at: Option<DateTime>,
    pub created_at: DateTime,
}
impl Default for AuthToken {
    fn default() -> Self {
        Self {
            id: 0,
            user_id: 0,
            token: "".to_string(),
            active: false,
            expires_at: None,
            created_at: DateTime::default(),
        }
    }
}
impl ReferencesUser for AuthToken {
    fn user_id(&self) -> i32 {
        self.user_id
    }

    async fn get_owned_by_user(user_id: i32, database: &PgPool) -> Result<Vec<Self>, sqlx::Error>
    where
        Self: Sized,
    {
        let tokens = sqlx::query_as(
            r#"SELECT * FROM auth_tokens WHERE user_id = $1 ORDER BY created_at DESC"#,
        )
        .bind(user_id);
        tokens.fetch_all(database).await
    }
}
impl AuthToken {
    pub async fn insert(self, database: &PgPool) -> sqlx::Result<AuthToken> {
        let Self {
            user_id,
            token,
            active,
            expires_at,
            created_at,
            ..
        } = self;
        let token =  sqlx::query_as(
            r#"INSERT INTO auth_tokens (user_id, token, active, expires_at, created_at) VALUES ($1, $2, $3, $4, $4) RETURNING *"#,
        ).bind(user_id)
        .bind(token)
        .bind(active)
        .bind(expires_at)
        .bind(created_at)
        .fetch_one(database).await?;
        Ok(token)
    }
    pub async fn get_by_token(token: &str, database: &PgPool) -> sqlx::Result<Option<Self>> {
        let token = sqlx::query_as(r#"SELECT * FROM auth_tokens WHERE token = $1"#)
            .bind(token)
            .fetch_optional(database)
            .await?;
        Ok(token)
    }
}

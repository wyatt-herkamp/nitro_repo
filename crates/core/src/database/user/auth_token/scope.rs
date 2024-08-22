use serde::{Deserialize, Serialize};
use sqlx::{prelude::FromRow, PgPool};

use crate::{database::DateTime, user::scopes::Scopes};

/// Table Name: user_auth_token_scopes

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, FromRow)]
pub struct AuthTokenScope {
    pub id: i32,
    pub user_auth_token_id: i32,
    pub scope: Scopes,
    pub created_at: DateTime,
}
#[derive(Debug)]
pub struct NewAuthTokenScope {
    pub user_auth_token_id: i32,
    pub scope: Scopes,
}
impl NewAuthTokenScope {
    pub async fn insert_no_return(&self, database: &PgPool) -> sqlx::Result<()> {
        sqlx::query(
            r#"INSERT INTO user_auth_token_scopes (user_auth_token_id, scope) VALUES ($1, $2)"#,
        )
        .bind(self.user_auth_token_id)
        .bind(self.scope)
        .execute(database)
        .await?;
        Ok(())
    }
}

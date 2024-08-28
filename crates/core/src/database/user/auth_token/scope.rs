use serde::{Deserialize, Serialize};
use sqlx::{prelude::FromRow, PgPool};
use utoipa::ToSchema;

use crate::{database::DateTime, user::scopes::NRScope};

/// Table Name: user_auth_token_scopes

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, FromRow, ToSchema)]
pub struct AuthTokenScope {
    pub id: i32,
    pub user_auth_token_id: i32,
    pub scope: NRScope,
    pub created_at: DateTime,
}
impl AuthTokenScope {
    pub async fn get_by_token_id(
        user_auth_token_id: i32,
        database: &PgPool,
    ) -> sqlx::Result<Vec<Self>> {
        let scopes =
            sqlx::query_as(r#"SELECT * FROM user_auth_token_scopes WHERE user_auth_token_id = $1"#)
                .bind(user_auth_token_id)
                .fetch_all(database)
                .await?;
        Ok(scopes)
    }
}
#[derive(Debug)]
pub struct NewAuthTokenScope {
    pub user_auth_token_id: i32,
    pub scope: NRScope,
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

use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use tracing::instrument;
use utoipa::ToSchema;

use crate::database::entities::user::auth_token::{AuthTokenRepositoryScope, AuthTokenScope};
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, FromRow, ToSchema)]
pub struct AuthTokenResponse {
    pub id: i32,
    pub user_id: i32,
    pub name: Option<String>,
    pub description: Option<String>,
    pub active: bool,
    pub source: String,
    pub expires_at: Option<chrono::DateTime<chrono::FixedOffset>>,
    pub created_at: chrono::DateTime<chrono::FixedOffset>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct AuthTokenFullResponse {
    pub token: AuthTokenResponse,
    pub scopes: Vec<AuthTokenScope>,
    pub repository_scopes: Vec<AuthTokenRepositoryScope>,
}
impl AuthTokenFullResponse {
    #[instrument(name = "AuthTokenFullResponse::get_by_id_and_user_id")]
    pub async fn find_by_id_and_user_id(
        id: i32,
        user_id: i32,
        database: &sqlx::PgPool,
    ) -> Result<Option<AuthTokenFullResponse>, sqlx::Error> {
        let Some(base) = sqlx::query_as::<_, AuthTokenResponse>(
            r#"SELECT * FROM user_auth_tokens WHERE id = $1 user_id = $2"#,
        )
        .bind(id)
        .bind(user_id)
        .fetch_optional(database)
        .await?
        else {
            return Ok(None);
        };
        let (scopes, repository_scopes) =
            AuthTokenFullResponse::get_scopes_for(base.id, database).await?;
        Ok(Some(AuthTokenFullResponse {
            token: base,
            scopes,
            repository_scopes,
        }))
    }
    pub async fn find_by_id(
        id: i32,
        database: &sqlx::PgPool,
    ) -> Result<Option<AuthTokenFullResponse>, sqlx::Error> {
        let Some(base) = sqlx::query_as::<_, AuthTokenResponse>(
            r#"SELECT * FROM user_auth_tokens WHERE id = $1"#,
        )
        .bind(id)
        .fetch_optional(database)
        .await?
        else {
            return Ok(None);
        };
        let (scopes, repository_scopes) =
            AuthTokenFullResponse::get_scopes_for(base.id, database).await?;
        Ok(Some(AuthTokenFullResponse {
            token: base,
            scopes,
            repository_scopes,
        }))
    }
    #[instrument(name = "AuthTokenFullResponse::get_all_for_user")]
    pub async fn get_all_for_user(
        user_id: i32,
        database: &sqlx::PgPool,
    ) -> Result<Vec<AuthTokenFullResponse>, sqlx::Error> {
        let tokens = sqlx::query_as::<_, AuthTokenResponse>(
            r#"SELECT * FROM user_auth_tokens WHERE user_id = $1"#,
        )
        .bind(user_id)
        .fetch_all(database)
        .await?;
        let mut responses = Vec::with_capacity(tokens.len());
        for token in tokens {
            let (scopes, repository_scopes) =
                AuthTokenFullResponse::get_scopes_for(token.id, database).await?;
            responses.push(AuthTokenFullResponse {
                token,
                scopes,
                repository_scopes,
            });
        }
        Ok(responses)
    }

    #[instrument(name = "AuthTokenFullResponse::get_scopes")]
    pub async fn get_scopes_for(
        id: i32,
        database: &sqlx::PgPool,
    ) -> Result<(Vec<AuthTokenScope>, Vec<AuthTokenRepositoryScope>), sqlx::Error> {
        let scopes = AuthTokenScope::get_by_token_id(id, database).await?;
        let repository_scopes = AuthTokenRepositoryScope::get_by_token_id(id, database).await?;
        Ok((scopes, repository_scopes))
    }
}

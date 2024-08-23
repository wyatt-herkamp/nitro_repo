use serde::{Deserialize, Serialize};
use sqlx::{prelude::FromRow, PgPool};
use tracing::instrument;
use uuid::Uuid;

use crate::{
    database::DateTime,
    user::{permissions::RepositoryActions, scopes::Scopes},
};

use super::ReferencesUser;
mod repository_scope;
mod scope;
mod utils;
pub use repository_scope::*;
pub use scope::*;
pub use utils::*;
/// Table Name: user_auth_tokens
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, FromRow)]
pub struct AuthToken {
    pub id: i32,
    pub user_id: i32,
    pub token: String,
    pub active: bool,
    pub expires_at: Option<DateTime>,
    pub created_at: DateTime,
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
            r#"SELECT * FROM user_auth_tokens WHERE user_id = $1 ORDER BY created_at DESC"#,
        )
        .bind(user_id);
        tokens.fetch_all(database).await
    }
}
impl AuthToken {
    pub async fn get_by_token(token: &str, database: &PgPool) -> sqlx::Result<Option<Self>> {
        let token =
            sqlx::query_as(r#"SELECT * FROM user_auth_tokens WHERE token = $1 AND active = true"#)
                .bind(hash_token(token))
                .fetch_optional(database)
                .await?;
        Ok(token)
    }
    pub async fn has_scope(&self, scope: Scopes, database: &PgPool) -> sqlx::Result<bool> {
        let can_read: i64 = sqlx::query_scalar(
            r#"SELECT COUNT(id) FROM user_auth_token_scopes WHERE user_auth_token_id = $1 AND scope = $2"#,
        )
        .bind(self.id)
        .bind(scope)
        .fetch_one(database).await?;
        Ok(can_read > 0)
    }
    pub async fn get_scopes(&self, database: &PgPool) -> sqlx::Result<Vec<AuthTokenScope>> {
        let scopes =
            sqlx::query_as(r#"SELECT * FROM user_auth_token_scopes WHERE user_auth_token_id = $1"#)
                .bind(self.id)
                .fetch_all(database)
                .await?;
        Ok(scopes)
    }
    /// Checks if the user has the general scope for the repository action.
    /// If it will check if the user has the specific scope for the repository action
    #[instrument]
    pub async fn has_repository_action(
        &self,
        repository_id: Uuid,
        repository_action: RepositoryActions,
        database: &PgPool,
    ) -> sqlx::Result<bool> {
        // Check if the user has the general scope. See RepositoryActions for more info
        if self.has_scope(repository_action.into(), database).await? {
            // The user has the general scope for this action
            return Ok(true);
        }
        // TODO condense this into one query
        let Some(actions) = sqlx::query_scalar::<_, Vec<RepositoryActions>>(
            r#"SELECT actions FROM user_auth_token_repository_scopes WHERE user_auth_token_id = $1 AND repository_id = $2"#,
        )
        .bind(self.id)
        .bind(repository_id)
        .fetch_optional(database).await? else{
            return Ok(false);
        };
        Ok(actions.contains(&repository_action))
    }
}

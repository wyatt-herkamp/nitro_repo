use pg_extended_sqlx_queries::prelude::*;
use sqlx::{PgPool, prelude::FromRow};
use tracing::instrument;
use uuid::Uuid;

use super::ReferencesUser;
use crate::user::{permissions::RepositoryActions, scopes::NRScope};
mod repository_scope;
mod scope;
mod utils;
pub use repository_scope::*;
pub use scope::*;
pub use utils::*;
/// Table Name: user_auth_tokens
#[derive(Debug, Clone, PartialEq, Eq, FromRow, TableType)]
#[table(name = "user_auth_tokens")]
pub struct AuthToken {
    pub id: i32,
    pub user_id: i32,
    pub name: Option<String>,
    pub description: Option<String>,
    pub token: String,
    pub active: bool,
    pub source: String,
    pub expires_at: Option<chrono::DateTime<chrono::FixedOffset>>,
    /// When the token was last used to authenticate. See [`AuthToken::touch_last_used`].
    pub last_used_at: Option<chrono::DateTime<chrono::FixedOffset>>,
    pub created_at: chrono::DateTime<chrono::FixedOffset>,
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
    /// Looks a token up for authentication.
    ///
    /// The expiry check is the point. `expires_at` has existed on this table since the schema was
    /// written and was never consulted, so a token past its expiry authenticated exactly like a
    /// live one — the password reset table next door gets this right, which is what makes the
    /// omission here easy to miss.
    pub async fn get_by_token(token: &str, database: &PgPool) -> sqlx::Result<Option<Self>> {
        let token = sqlx::query_as(
            r#"SELECT * FROM user_auth_tokens
               WHERE token = $1 AND active = true
                 AND (expires_at IS NULL OR expires_at > NOW())"#,
        )
        .bind(hash_token(token))
        .fetch_optional(database)
        .await?;
        Ok(token)
    }
    /// Revokes every token belonging to a user.
    ///
    /// Returns how many were removed. This is the "revoke all tokens" the admin surface needs
    /// after a credential leak — there was no way to do it, for yourself or for anyone else.
    #[instrument(skip(database))]
    pub async fn delete_all_for_user(user_id: i32, database: &PgPool) -> sqlx::Result<u64> {
        let result = sqlx::query(r#"DELETE FROM user_auth_tokens WHERE user_id = $1"#)
            .bind(user_id)
            .execute(database)
            .await?;
        Ok(result.rows_affected())
    }
    /// Records that a token was just used.
    ///
    /// Written at most once an hour per token: this runs on every authenticated request, and a
    /// write per request would turn a read-only API call into a write against a hot row. An hour
    /// is enough to answer "is this token still in use", which is what the profile page shows it
    /// for.
    pub async fn touch_last_used(&self, database: &PgPool) -> sqlx::Result<()> {
        sqlx::query(
            r#"UPDATE user_auth_tokens SET last_used_at = NOW()
               WHERE id = $1
                 AND (last_used_at IS NULL OR last_used_at < NOW() - INTERVAL '1 hour')"#,
        )
        .bind(self.id)
        .execute(database)
        .await?;
        Ok(())
    }
    pub async fn has_scope(&self, scope: NRScope, database: &PgPool) -> sqlx::Result<bool> {
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
    pub async fn get_by_id_and_user_id(
        id: i32,
        user_id: i32,
        database: &PgPool,
    ) -> sqlx::Result<Option<Self>> {
        let token =
            sqlx::query_as(r#"SELECT * FROM user_auth_tokens WHERE id = $1 AND user_id = $2"#)
                .bind(id)
                .bind(user_id)
                .fetch_optional(database)
                .await?;
        Ok(token)
    }
    pub async fn delete(&self, database: &PgPool) -> sqlx::Result<()> {
        sqlx::query(r#"DELETE FROM user_auth_tokens WHERE id = $1"#)
            .bind(self.id)
            .execute(database)
            .await?;
        Ok(())
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct NewAuthToken {
    pub user_id: i32,
    pub name: Option<String>,
    pub description: Option<String>,
    pub source: String,
    pub scopes: Vec<NRScope>,
    pub repositories: Vec<(Uuid, Vec<RepositoryActions>)>,
    /// When the token stops working. `None` means never.
    ///
    /// There was no way to set this — the column existed, the insert never wrote it, and the
    /// profile UI showed a disabled field reading "Not implemented".
    pub expires_at: Option<chrono::DateTime<chrono::FixedOffset>>,
}
impl NewAuthToken {
    pub async fn insert(self, database: &PgPool) -> sqlx::Result<(i32, String)> {
        let (token, hashed_token) = create_token(database).await?;
        let Self {
            user_id,
            name,
            description,
            source,
            scopes,
            repositories,
            expires_at,
        } = self;

        let token_id: i32 = sqlx::query_scalar(
            r#"INSERT INTO user_auth_tokens (user_id, name, description, token, source, expires_at) VALUES ($1, $2, $3, $4, $5, $6) RETURNING id"#,
        )
        .bind(user_id)
        .bind(name)
        .bind(description)
        .bind(hashed_token)
        .bind(source)
        .bind(expires_at)
        .fetch_one(database)
        .await?;

        for scope in scopes {
            let scope = NewAuthTokenScope {
                user_auth_token_id: token_id,
                scope,
            };
            scope.insert_no_return(database).await?;
        }

        for (repository, actions) in repositories {
            let repository_scope = NewRepositoryScope {
                token_id,
                repository,
                actions,
            };
            repository_scope.insert_no_return(database).await?;
        }

        Ok((token_id, token))
    }
}

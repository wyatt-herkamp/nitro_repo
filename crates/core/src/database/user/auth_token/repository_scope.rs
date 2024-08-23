use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use sqlx::{
    prelude::{FromRow, Type},
    PgPool,
};
use tracing::{debug, instrument, span};
use uuid::Uuid;

use crate::{database::DateTime, user::permissions::RepositoryActions};

use super::{create_token, hash_token};
/// Represents the actions that can be taken on a repository
///
/// Repository Scopes can be overridden by having a scope for all repositories
///
/// RepositoryActions::Read has Scopes::ReadRepository meaning they can read all repositories that the user has access to
/// RepositoryActions::Write has Scopes::WriteRepository meaning they can write to all repositories that the user has access to

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, FromRow)]
pub struct AuthTokenRepositoryScope {
    pub id: i32,
    pub user_auth_token_id: i32,
    pub repository_id: Uuid,
    pub action: Vec<RepositoryActions>,
    pub created_at: DateTime,
}

#[derive(Debug, Clone, PartialEq, Eq, Builder)]
pub struct NewRepositoryToken {
    pub user_id: i32,
    pub source: String,
    pub repositories: Vec<(Uuid, Vec<RepositoryActions>)>,
    pub expires_at: Option<DateTime>,
}
impl NewRepositoryToken {
    pub fn new(
        user_id: i32,
        source: String,
        repository: Uuid,
        actions: Vec<RepositoryActions>,
    ) -> Self {
        Self {
            user_id,
            source,
            repositories: vec![(repository, actions)],
            expires_at: None,
        }
    }
    pub fn add_repository(mut self, repository: Uuid, actions: Vec<RepositoryActions>) -> Self {
        self.repositories.push((repository, actions));
        self
    }
    #[instrument]
    pub async fn insert(self, database: &PgPool) -> sqlx::Result<(i32, String)> {
        let token = create_token(database).await?;
        let hashed_token = hash_token(&token);
        let Self {
            user_id,
            source,
            repositories,
            expires_at,
        } = self;

        let token_id: i32 =  sqlx::query_scalar(
            r#"INSERT INTO user_auth_tokens (user_id, token, source, expires_at) VALUES ($1, $2, $3, $4) RETURNING id"#,
        ).bind(user_id)
        .bind(hashed_token)
        .bind(source)
        .bind(expires_at)
        .fetch_one(database).await?;
        let span = span!(tracing::Level::DEBUG, "inserting scopes");
        let _guard = span.enter();
        for (repository_id, actions) in repositories {
            debug!(?repository_id, ?actions, "Inserting scope");
            NewRepositoryScope {
                token_id: token_id,
                repository: repository_id,
                actions,
            }
            .insert_no_return(database)
            .await?;
        }
        Ok((token_id, token))
    }
}
#[derive(Debug)]
pub struct NewRepositoryScope {
    pub token_id: i32,
    pub repository: Uuid,
    pub actions: Vec<RepositoryActions>,
}
impl NewRepositoryScope {
    #[instrument]
    pub async fn insert_no_return(self, database: &PgPool) -> sqlx::Result<()> {
        let Self {
            token_id,
            repository,
            actions,
        } = self;
        sqlx::query(
                r#"INSERT INTO user_auth_token_repository_scopes (user_auth_token_id, repository_id, actions) VALUES ($1, $2, $3)"#,
            )
            .bind(token_id)
            .bind(repository)
            .bind(actions)
            .execute(database)
            .await?;

        Ok(())
    }
}

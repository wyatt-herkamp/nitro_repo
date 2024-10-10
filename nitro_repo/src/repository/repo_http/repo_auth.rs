use axum::{
    async_trait,
    extract::{FromRef, FromRequestParts},
};
use http::request::Parts;
use nr_core::{
    database::user::{auth_token::AuthToken, UserSafeData, UserType},
    user::permissions::{
        does_user_and_token_have_repository_action, HasPermissions, RepositoryActions,
        UserPermissions,
    },
};
use sqlx::PgPool;
use strum::EnumIs;
use tracing::{debug, instrument};
use uuid::Uuid;

use crate::app::{
    authentication::{session::Session, verify_login, AuthenticationError, AuthenticationRaw},
    NitroRepo,
};

#[derive(Clone, Debug, PartialEq, EnumIs)]
pub enum RepositoryAuthentication {
    /// An Auth Token was passed under the Authorization Header
    AuthToken(AuthToken, UserSafeData),
    /// Uses a Session Cookie or Session Header
    Session(Session, UserSafeData),
    /// Uses Basic Authorization Header
    Basic(Option<AuthToken>, UserSafeData),
    /// An authorization header was passed but it does not match any known types
    Other(String, String),
    /// No Identification was passed
    NoIdentification,
}
impl RepositoryAuthentication {
    #[instrument]
    pub async fn can_access_repository(
        &self,
        action: RepositoryActions,
        repository_id: Uuid,
        database: &PgPool,
    ) -> Result<bool, AuthenticationError> {
        match self {
            RepositoryAuthentication::AuthToken(token, user)
            | RepositoryAuthentication::Basic(Some(token), user) => {
                debug!("Request has an Auth Token. Checking if it has access to the repository");
                does_user_and_token_have_repository_action(
                    user,
                    token,
                    action,
                    repository_id,
                    database,
                )
                .await
                .map_err(AuthenticationError::from)
            }
            RepositoryAuthentication::Session(_, user)
            | RepositoryAuthentication::Basic(None, user) => {
                Ok(user.has_action(action, repository_id, database).await?)
            }
            _ => Ok(false),
        }
    }
    #[instrument]
    pub async fn get_user_if_has_action(
        &self,
        action: RepositoryActions,
        repository_id: Uuid,
        database: &PgPool,
    ) -> Result<Option<&UserSafeData>, AuthenticationError> {
        match self {
            RepositoryAuthentication::AuthToken(token, user)
            | RepositoryAuthentication::Basic(Some(token), user) => {
                debug!("Request has an Auth Token. Checking if it has access to the repository");
                if does_user_and_token_have_repository_action(
                    user,
                    token,
                    action,
                    repository_id,
                    database,
                )
                .await?
                {
                    Ok(Some(user))
                } else {
                    Ok(None)
                }
            }
            RepositoryAuthentication::Session(_, user)
            | RepositoryAuthentication::Basic(None, user) => {
                if user.has_action(action, repository_id, database).await? {
                    Ok(Some(user))
                } else {
                    Ok(None)
                }
            }
            _ => Ok(None),
        }
    }
}
impl HasPermissions for RepositoryAuthentication {
    fn get_permissions(&self) -> Option<UserPermissions> {
        match self {
            RepositoryAuthentication::AuthToken(_, user) => user.get_permissions(),
            RepositoryAuthentication::Session(_, user) => user.get_permissions(),
            RepositoryAuthentication::Basic(_, user) => user.get_permissions(),
            _ => None,
        }
    }

    fn user_id(&self) -> Option<i32> {
        match self {
            RepositoryAuthentication::AuthToken(_, user) => Some(user.id),
            RepositoryAuthentication::Session(_, user) => Some(user.id),
            RepositoryAuthentication::Basic(_, user) => Some(user.id),
            _ => None,
        }
    }
}
impl RepositoryAuthentication {
    pub fn get_user_id(&self) -> Option<i32> {
        match self {
            RepositoryAuthentication::AuthToken(_, user) => Some(user.id),
            RepositoryAuthentication::Session(_, user) => Some(user.id),
            RepositoryAuthentication::Basic(_, user) => Some(user.id),
            _ => None,
        }
    }
    pub fn get_user(&self) -> Option<&UserSafeData> {
        match self {
            RepositoryAuthentication::AuthToken(_, user) => Some(user),
            RepositoryAuthentication::Session(_, user) => Some(user),
            RepositoryAuthentication::Basic(_, user) => Some(user),
            _ => None,
        }
    }
    pub fn has_auth_token(&self) -> bool {
        matches!(
            self,
            RepositoryAuthentication::AuthToken(..) | RepositoryAuthentication::Basic(Some(_), _)
        )
    }
}
#[async_trait]
impl<S> FromRequestParts<S> for RepositoryAuthentication
where
    NitroRepo: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = AuthenticationError;
    #[instrument(
        name = "repository_auth_from_request",
        skip(parts, state),
        fields(project_module = "Authentication")
    )]
    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let raw_extension = parts.extensions.get::<AuthenticationRaw>().cloned();
        let repo = NitroRepo::from_ref(state);
        let Some(raw_auth) = raw_extension else {
            return Err(AuthenticationError::Unauthorized);
        };
        match raw_auth {
            AuthenticationRaw::AuthToken(token) => {
                let (token, user) = get_by_auth_token(&token, &repo.database).await?;
                Ok(RepositoryAuthentication::Basic(Some(token), user))
            }
            AuthenticationRaw::Session(session) => {
                let user = UserSafeData::get_by_id(session.user_id, &repo.database)
                    .await?
                    .ok_or(AuthenticationError::Unauthorized)?;
                Ok(RepositoryAuthentication::Session(session, user))
            }
            AuthenticationRaw::Basic { username, password } => {
                match verify_login(username, &password, &repo.database).await {
                    Ok(user) => Ok(RepositoryAuthentication::Basic(None, user)),
                    Err(AuthenticationError::Unauthorized) => {
                        let (token, user) = get_by_auth_token(&password, &repo.database).await?;
                        Ok(RepositoryAuthentication::Basic(Some(token), user))
                    }
                    Err(err) => Err(err),
                }
            }
            AuthenticationRaw::NoIdentification => Ok(RepositoryAuthentication::NoIdentification),
            AuthenticationRaw::AuthorizationHeaderUnknown(scheme, value) => {
                debug!("Unknown Authorization Header: {} {}", scheme, value);
                return Ok(RepositoryAuthentication::Other(scheme, value));
            }
        }
    }
}
async fn get_by_auth_token(
    token: &str,
    database: &PgPool,
) -> Result<(AuthToken, UserSafeData), AuthenticationError> {
    let token = AuthToken::get_by_token(token, database)
        .await?
        .ok_or(AuthenticationError::Unauthorized)?;
    let user = UserSafeData::get_by_id(token.user_id, database)
        .await?
        .ok_or(AuthenticationError::Unauthorized)?;
    Ok((token, user))
}

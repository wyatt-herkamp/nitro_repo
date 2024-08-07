use std::fmt::Debug;
use std::ops::Deref;

use argon2::{Argon2, PasswordHash, PasswordVerifier};
use axum::async_trait;
use axum::extract::{FromRef, FromRequestParts};
use axum::response::IntoResponse;
use axum_extra::extract::cookie::Cookie;
use derive_more::From;

use http::request::Parts;
use http::Response;
use nr_core::database::user::auth_token::AuthToken;
use nr_core::database::user::{UserModel, UserSafeData, UserType};
use nr_core::user::permissions::{HasPermissions, UserPermissions};
use serde::Serialize;
use session::Session;
use sqlx::PgPool;
use thiserror::Error;
use tracing::{error, warn};
use utoipa::ToSchema;

use crate::utils::headers::AuthorizationHeader;

use super::NitroRepo;

pub mod api_middleware;
pub mod session;

#[derive(Error, Debug)]
pub enum AuthenticationError {
    #[error("DB error {0}")]
    DBError(#[from] sqlx::Error),
    #[error("You are not logged in.")]
    Unauthorized,
    #[error("Password is not able to be verified.")]
    PasswordVerificationError,
}
impl IntoResponse for AuthenticationError {
    fn into_response(self) -> axum::response::Response {
        error!("{}", self);
        Response::builder()
            .status(http::StatusCode::UNAUTHORIZED)
            .body("Unauthorized".into())
            .unwrap()
    }
}
#[derive(Clone, Debug, PartialEq)]
pub enum Authentication {
    /// An Auth Token was passed under the Authorization Header
    AuthToken(AuthToken, UserSafeData),
    /// Session Value from Cookie
    Session(Session, UserSafeData),
}
impl Deref for Authentication {
    type Target = UserSafeData;
    fn deref(&self) -> &Self::Target {
        match self {
            Authentication::AuthToken(_, user) => user,
            Authentication::Session(_, user) => user,
        }
    }
}
impl HasPermissions for Authentication {
    fn get_permissions(&self) -> Option<&UserPermissions> {
        match self {
            Authentication::AuthToken(_, user) => user.get_permissions(),
            Authentication::Session(_, user) => user.get_permissions(),
        }
    }
}
#[async_trait]
impl<S> FromRequestParts<S> for Authentication
where
    NitroRepo: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = AuthenticationError;
    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let raw_extension = parts.extensions.get::<AuthenticationRaw>().cloned();
        let repo = NitroRepo::from_ref(state);
        let Some(raw_auth) = raw_extension else {
            return Err(AuthenticationError::Unauthorized);
        };
        let auth = match raw_auth {
            AuthenticationRaw::NoIdentification => {
                return Err(AuthenticationError::Unauthorized);
            }
            AuthenticationRaw::AuthToken(..) => {
                todo!("Auth Token Authentication")
            }
            AuthenticationRaw::Session(session) => {
                let user = UserSafeData::get_by_id(session.user_id, &repo.database)
                    .await
                    .map_err(AuthenticationError::DBError)?
                    .ok_or(AuthenticationError::Unauthorized)?;
                Authentication::Session(session, user)
            }
            AuthenticationRaw::AuthorizationHeaderUnknown(scheme, value) => {
                error!("Unknown Authorization Header: {} {}", scheme, value);
                return Err(AuthenticationError::Unauthorized);
            }
            AuthenticationRaw::Basic { .. } => {
                warn!("Basic Auth is not allowed in API routes");
                return Err(AuthenticationError::Unauthorized);
            }
        };
        Ok(auth)
    }
}

#[derive(Debug, Serialize, Clone, From, ToSchema)]
pub struct MeWithSession {
    session: Session,
    user: UserSafeData,
}

#[derive(Clone, Debug, PartialEq)]
pub enum RepositoryAuthentication {
    AuthToken(AuthToken, UserSafeData),
    Session(Session, UserSafeData),
    Basic(UserSafeData, Option<AuthToken>),
    Other(String, String),
    NoIdentification,
}
impl HasPermissions for RepositoryAuthentication {
    fn get_permissions(&self) -> Option<&UserPermissions> {
        match self {
            RepositoryAuthentication::AuthToken(_, user) => user.get_permissions(),
            RepositoryAuthentication::Session(_, user) => user.get_permissions(),
            RepositoryAuthentication::Basic(user, _) => user.get_permissions(),
            _ => None,
        }
    }
}
impl RepositoryAuthentication {
    pub fn get_user(&self) -> Option<&UserSafeData> {
        match self {
            RepositoryAuthentication::AuthToken(_, user) => Some(user),
            RepositoryAuthentication::Session(_, user) => Some(user),
            RepositoryAuthentication::Basic(user, _) => Some(user),
            _ => None,
        }
    }
    pub fn has_auth_token(&self) -> bool {
        match self {
            RepositoryAuthentication::AuthToken(..) => true,
            RepositoryAuthentication::Basic(_, token) => token.is_some(),
            _ => false,
        }
    }
    pub fn check_permissions<F>(&self, f: F) -> bool
    where
        F: FnOnce(&UserPermissions) -> bool,
    {
        match self {
            RepositoryAuthentication::AuthToken(_, user) => f(&user.permissions),
            RepositoryAuthentication::Session(_, user) => f(&user.permissions),
            RepositoryAuthentication::Basic(user, _) => f(&user.permissions),
            _ => false,
        }
    }
}
#[async_trait]
impl<S> FromRequestParts<S> for RepositoryAuthentication
where
    NitroRepo: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = AuthenticationError;
    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let raw_extension = parts.extensions.get::<AuthenticationRaw>().cloned();
        let repo = NitroRepo::from_ref(state);
        let Some(raw_auth) = raw_extension else {
            return Err(AuthenticationError::Unauthorized);
        };
        let auth = match raw_auth {
            AuthenticationRaw::NoIdentification => {
                return Err(AuthenticationError::Unauthorized);
            }
            AuthenticationRaw::AuthToken(..) => {
                todo!("Auth Token Authentication")
            }
            AuthenticationRaw::Session(session) => {
                let user = UserSafeData::get_by_id(session.user_id, &repo.database)
                    .await
                    .map_err(AuthenticationError::DBError)?
                    .ok_or(AuthenticationError::Unauthorized)?;
                RepositoryAuthentication::Session(session, user)
            }
            AuthenticationRaw::AuthorizationHeaderUnknown(scheme, value) => {
                error!("Unknown Authorization Header: {} {}", scheme, value);
                return Err(AuthenticationError::Unauthorized);
            }
            AuthenticationRaw::Basic { username, password } => {
                let user = verify_login(username, password, &repo.database).await?;
                // TODO: Check if it is an API Token and not a username/password
                RepositoryAuthentication::Basic(user, None)
            }
        };
        Ok(auth)
    }
}

#[derive(Debug, Clone)]
pub enum AuthenticationRaw {
    /// No Authorization Header was passed.API Routes will most likely reject this
    /// Repository Routes will accept if it is allowed
    NoIdentification,
    /// An Auth Token was passed under the Authorization Header
    AuthToken(String),
    /// Session Value from Cookie
    Session(Session),
    /// If the Authorization Header could not be parsed. Give them the value
    AuthorizationHeaderUnknown(String, String),
    /// Authorization Basic Header
    Basic { username: String, password: String },
}
impl AuthenticationRaw {
    pub fn new_from_header(header: AuthorizationHeader, site: &NitroRepo) -> Self {
        match header {
            AuthorizationHeader::Basic { username, password } => {
                AuthenticationRaw::Basic { username, password }
            }
            AuthorizationHeader::Bearer { token } => AuthenticationRaw::AuthToken(token),
            AuthorizationHeader::Session { session } => {
                let session = match site.session_manager.get_session(&session) {
                    Ok(Some(ok)) => AuthenticationRaw::Session(ok),
                    Err(err) => {
                        error!("Failed to get session: {}", err);
                        AuthenticationRaw::NoIdentification
                    }
                    Ok(None) => AuthenticationRaw::NoIdentification,
                };
                session
            }
            AuthorizationHeader::Other { scheme, value } => {
                AuthenticationRaw::AuthorizationHeaderUnknown(scheme, value)
            }
        }
    }
    pub fn new_from_cookie(cookie: &Cookie<'static>, site: &NitroRepo) -> Self {
        let session = match site.session_manager.get_session(cookie.value()) {
            Ok(Some(ok)) => AuthenticationRaw::Session(ok),
            Err(err) => {
                error!("Failed to get session: {}", err);
                AuthenticationRaw::NoIdentification
            }
            Ok(None) => AuthenticationRaw::NoIdentification,
        };
        session
    }
}
#[inline(always)]
pub async fn verify_login(
    username: impl AsRef<str>,
    password: impl AsRef<str>,
    database: &PgPool,
) -> Result<UserSafeData, AuthenticationError> {
    let user_found: Option<UserModel> = UserModel::get_by_username_or_email(username, database)
        .await
        .map_err(AuthenticationError::DBError)?;
    if user_found.is_none() {
        return Err(AuthenticationError::Unauthorized);
    }
    let argon2 = Argon2::default();
    let user = user_found.unwrap();
    let Some(parsed_hash) = user
        .password
        .as_ref()
        .map(|x| PasswordHash::new(x))
        .transpose()
        .map_err(|err| {
            error!("Failed to parse password hash: {}", err);
            AuthenticationError::PasswordVerificationError
        })?
    else {
        return Err(AuthenticationError::Unauthorized);
    };

    if argon2
        .verify_password(password.as_ref().as_bytes(), &parsed_hash)
        .is_err()
    {
        return Err(AuthenticationError::Unauthorized);
    }
    Ok(user.into())
}

pub async fn get_user_and_auth_token(
    token: &str,
    database: &PgPool,
) -> Result<(UserSafeData, AuthToken), AuthenticationError> {
    let auth_token = AuthToken::get_by_token(token, database)
        .await
        .map_err(AuthenticationError::DBError)?
        .ok_or(AuthenticationError::Unauthorized)?;
    let user = UserSafeData::get_by_id(auth_token.user_id, database)
        .await
        .map_err(AuthenticationError::DBError)?
        .ok_or(AuthenticationError::Unauthorized)?;
    Ok((user, auth_token))
}

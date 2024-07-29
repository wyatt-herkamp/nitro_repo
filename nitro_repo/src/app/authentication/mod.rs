use std::fmt::Debug;

use actix_web::dev::Payload;
use actix_web::{FromRequest, HttpMessage, HttpRequest};
use argon2::{Argon2, PasswordHash, PasswordVerifier};
use derive_more::From;
use futures::future::LocalBoxFuture;

use nr_core::database::user::auth_token::AuthToken;
use nr_core::database::user::{UserModel, UserSafeData, UserType};
use nr_core::user::permissions::HasPermissions;
use serde::Serialize;
use session::Session;
use sqlx::PgPool;
use this_actix_error::ActixError;
use thiserror::Error;
use tracing::error;

use super::DatabaseConnection;

pub mod api_middleware;
pub mod session;

#[derive(Error, Debug, ActixError)]
pub enum AuthenticationError {
    #[error("DB error {0}")]
    #[status_code(500)]
    DBError(#[from] sqlx::Error),
    #[error("You are not logged in.")]
    #[status_code(401)]
    Unauthorized,
    #[error("Password is not able to be verified.")]
    #[status_code(401)]
    PasswordVerificationError,
}
#[derive(Clone, Debug, PartialEq)]
pub enum Authentication {
    /// An Auth Token was passed under the Authorization Header
    AuthToken(AuthToken, UserSafeData),
    /// Session Value from Cookie
    Session(Session, UserSafeData),
    /// Authorization Basic Header
    Basic(UserSafeData),
}
impl HasPermissions for Authentication {
    fn get_permissions(&self) -> &nr_core::user::permissions::UserPermissions {
        match self {
            Authentication::AuthToken(_, user) => &user.permissions,
            Authentication::Session(_, user) => &user.permissions,
            Authentication::Basic(user) => &user.permissions,
        }
    }
}
#[derive(Debug, Serialize, Clone, From)]
pub struct MeWithSession {
    session: Session,
    user: UserSafeData,
}
impl FromRequest for Authentication {
    type Error = AuthenticationError;
    type Future = LocalBoxFuture<'static, Result<Self, AuthenticationError>>;

    #[inline]
    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        let model = req.extensions_mut().get::<AuthenticationRaw>().cloned();
        if let Some(model) = model {
            let database = req.app_data::<DatabaseConnection>().unwrap().clone();
            return Box::pin(async move {
                match model {
                    AuthenticationRaw::Session(session) => {
                        let user = UserSafeData::get_by_id(session.user_id, database.as_ref())
                            .await?
                            .ok_or(AuthenticationError::Unauthorized)?;
                        Ok(Authentication::Session(session, user))
                    }
                    AuthenticationRaw::AuthToken(token) => {
                        let (user, auth_token) =
                            get_user_and_auth_token(&token, database.as_ref()).await?;
                        Ok(Authentication::AuthToken(auth_token, user))
                    }
                    // TODO: Implement AuthToken, Basic
                    _ => Err(AuthenticationError::Unauthorized),
                }
            });
        }
        Box::pin(async move { Err(AuthenticationError::Unauthorized) })
    }
}
#[derive(Clone, Debug, PartialEq)]
pub enum RepositoryAuthentication {
    AuthToken(AuthToken, UserSafeData),
    Session(Session, UserSafeData),
    Basic(UserSafeData),
    Other(String, String),
    NoIdentification,
}

impl FromRequest for RepositoryAuthentication {
    type Error = AuthenticationError;
    type Future = LocalBoxFuture<'static, Result<Self, AuthenticationError>>;

    #[inline]
    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        let model = req.extensions_mut().get::<AuthenticationRaw>().cloned();
        if let Some(model) = model {
            let database = req.app_data::<DatabaseConnection>().unwrap().clone();
            return Box::pin(async move {
                match model {
                    AuthenticationRaw::Session(session) => {
                        let user = UserSafeData::get_by_id(session.user_id, database.as_ref())
                            .await?
                            .ok_or(AuthenticationError::Unauthorized)?;
                        Ok(RepositoryAuthentication::Session(session, user))
                    }
                    AuthenticationRaw::AuthToken(token) => {
                        let (user, auth_token) =
                            get_user_and_auth_token(&token, database.as_ref()).await?;
                        Ok(RepositoryAuthentication::AuthToken(auth_token, user))
                    }
                    _ => Err(AuthenticationError::Unauthorized),
                }
            });
        }
        Box::pin(async move { Err(AuthenticationError::Unauthorized) })
    }
}

#[derive(Debug, Clone)]
pub enum AuthenticationRaw {
    /// Neither a Session or Auth Token exist.
    /// Might deny these requests in the future on API routes
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

#[inline(always)]
pub async fn verify_login(
    username: impl AsRef<str>,
    password: impl AsRef<str>,
    database: &DatabaseConnection,
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

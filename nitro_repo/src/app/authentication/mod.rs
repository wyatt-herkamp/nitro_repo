use std::fmt::{Debug, Display};
use std::ops::Deref;

use axum::async_trait;
use axum::body::Body;
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
use strum::EnumIs;
use thiserror::Error;
use tracing::{error, instrument, warn};
use utoipa::ToSchema;

use crate::error::IntoErrorResponse;
use crate::utils::headers::AuthorizationHeader;

use super::NitroRepo;

pub mod api_middleware;
pub mod session;

#[derive(Error, Debug)]
pub enum AuthenticationError {
    #[error("Error: {0}")]
    RequestTypeError(Box<dyn IntoErrorResponse>),
    #[error("You are not logged in.")]
    Unauthorized,
    #[error("Password is not able to be verified.")]
    PasswordVerificationError,
    #[error("No Auth Token Allowed here")]
    AuthTokenForbidden,
    #[error("Forbidden")]
    Forbidden,
}
impl From<sqlx::Error> for AuthenticationError {
    fn from(err: sqlx::Error) -> Self {
        AuthenticationError::RequestTypeError(Box::new(err))
    }
}

impl IntoErrorResponse for AuthenticationError {
    fn into_response_boxed(self: Box<Self>) -> axum::response::Response {
        self.into_response()
    }
}
impl IntoResponse for AuthenticationError {
    fn into_response(self) -> axum::response::Response {
        error!("Authentication Error: {}", self);
        match self {
            AuthenticationError::RequestTypeError(err) => err.into_response_boxed(),
            AuthenticationError::AuthTokenForbidden => Response::builder()
                .status(http::StatusCode::FORBIDDEN)
                .body(Body::from(
                    "This Route Prohibits Auth Tokens as a form of Authentication",
                ))
                .unwrap(),
            other => Response::builder()
                .status(http::StatusCode::UNAUTHORIZED)
                .body(Body::from(format!("Authentication Error: {}", other)))
                .unwrap(),
        }
    }
}
#[derive(Clone, Debug, PartialEq)]

pub struct OnlySessionAllowedAuthentication {
    pub user: UserSafeData,
    pub session: Session,
}
impl Deref for OnlySessionAllowedAuthentication {
    type Target = UserSafeData;
    fn deref(&self) -> &Self::Target {
        &self.user
    }
}

impl HasPermissions for OnlySessionAllowedAuthentication {
    fn user_id(&self) -> Option<i32> {
        Some(self.user.id)
    }

    fn get_permissions(&self) -> Option<UserPermissions> {
        self.user.get_permissions()
    }
}
#[async_trait]
impl<S> FromRequestParts<S> for OnlySessionAllowedAuthentication
where
    NitroRepo: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = AuthenticationError;
    #[instrument(
        name = "api_session_only_from_request",
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
            AuthenticationRaw::NoIdentification => {
                return Err(AuthenticationError::Unauthorized);
            }
            AuthenticationRaw::AuthToken(_) => {
                return Err(AuthenticationError::AuthTokenForbidden);
            }
            AuthenticationRaw::Session(session) => {
                let user = UserSafeData::get_by_id(session.user_id, &repo.database)
                    .await?
                    .ok_or(AuthenticationError::Unauthorized)?;
                return Ok(OnlySessionAllowedAuthentication { user, session });
            }
            other => {
                warn!("Unknown Authentication Method: {}", other);
                return Err(AuthenticationError::Unauthorized);
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, EnumIs)]
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
    fn user_id(&self) -> Option<i32> {
        match self {
            Authentication::AuthToken(_, user) => Some(user.id),
            Authentication::Session(_, user) => Some(user.id),
        }
    }

    fn get_permissions(&self) -> Option<UserPermissions> {
        match self {
            Authentication::AuthToken(_, user) | Authentication::Session(_, user) => {
                user.get_permissions()
            }
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
    #[instrument(
        name = "api_auth_from_request",
        skip(parts, state),
        fields(project_module = "Authentication")
    )]
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
            AuthenticationRaw::AuthToken(token) => {
                let (user, auth_token) = get_user_and_auth_token(&token, &repo.database).await?;
                Authentication::AuthToken(auth_token, user)
            }
            AuthenticationRaw::Session(session) => {
                let user = UserSafeData::get_by_id(session.user_id, &repo.database)
                    .await?
                    .ok_or(AuthenticationError::Unauthorized)?;
                Authentication::Session(session, user)
            }
            other => {
                warn!("Unknown Authentication Method: {}", other);
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
impl AsRef<str> for AuthenticationRaw {
    fn as_ref(&self) -> &str {
        match self {
            AuthenticationRaw::AuthToken(_) => "AuthToken",
            AuthenticationRaw::Session(_) => "Session",
            AuthenticationRaw::AuthorizationHeaderUnknown(_, _) => "AuthorizationHeaderUnknown",
            AuthenticationRaw::Basic { .. } => "Basic",
            AuthenticationRaw::NoIdentification => "NoIdentification",
        }
    }
}
impl Display for AuthenticationRaw {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let as_ref: &str = self.as_ref();
        write!(f, "{}", as_ref)
    }
}
impl AuthenticationRaw {
    #[instrument(skip(header, site), fields(project_module = "Authentication"))]
    pub fn new_from_header(header: AuthorizationHeader, site: &NitroRepo) -> Self {
        match header {
            AuthorizationHeader::Basic { username, password } => {
                AuthenticationRaw::Basic { username, password }
            }
            AuthorizationHeader::Bearer { token } => AuthenticationRaw::AuthToken(token),
            AuthorizationHeader::Session { session } => {
                match site.session_manager.get_session(&session) {
                    Ok(Some(ok)) => AuthenticationRaw::Session(ok),
                    Err(err) => {
                        error!("Failed to get session: {}", err);
                        AuthenticationRaw::NoIdentification
                    }
                    Ok(None) => AuthenticationRaw::NoIdentification,
                }
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
#[instrument(
    skip(username, password, database),
    fields(project_module = "Authentication")
)]
pub async fn verify_login(
    username: impl AsRef<str>,
    password: impl AsRef<str>,
    database: &PgPool,
) -> Result<UserSafeData, AuthenticationError> {
    let user_found: Option<UserModel> =
        UserModel::get_by_username_or_email(username, database).await?;
    let Some(user) = user_found else {
        return Err(AuthenticationError::Unauthorized);
    };
    password::verify_password(password.as_ref(), user.password.as_deref())?;
    Ok(user.into())
}

#[instrument(skip(token, database), fields(project_module = "Authentication"))]
pub async fn get_user_and_auth_token(
    token: &str,
    database: &PgPool,
) -> Result<(UserSafeData, AuthToken), AuthenticationError> {
    let auth_token = AuthToken::get_by_token(token, database)
        .await?
        .ok_or(AuthenticationError::Unauthorized)?;
    let user = UserSafeData::get_by_id(auth_token.user_id, database)
        .await?
        .ok_or(AuthenticationError::Unauthorized)?;
    Ok((user, auth_token))
}
pub mod password {
    use argon2::{
        password_hash::SaltString, Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
    };
    use rand::rngs::OsRng;
    use tracing::{error, instrument};

    use crate::app::authentication::AuthenticationError;
    #[instrument(skip(password), fields(project_module = "Authentication"))]
    pub fn encrypt_password(password: &str) -> Option<String> {
        let salt = SaltString::generate(&mut OsRng);

        let argon2 = Argon2::default();

        let password = argon2.hash_password(password.as_ref(), &salt);
        match password {
            Ok(ok) => Some(ok.to_string()),
            Err(err) => {
                error!("Failed to hash password: {}", err);
                None
            }
        }
    }
    #[instrument(skip(password, hash), fields(project_module = "Authentication"))]
    pub fn verify_password(password: &str, hash: Option<&str>) -> Result<(), AuthenticationError> {
        let argon2 = Argon2::default();
        let Some(parsed_hash) = hash.map(PasswordHash::new).transpose().map_err(|err| {
            error!("Failed to parse password hash: {}", err);
            AuthenticationError::PasswordVerificationError
        })?
        else {
            return Err(AuthenticationError::Unauthorized);
        };

        if argon2
            .verify_password(password.as_bytes(), &parsed_hash)
            .is_err()
        {
            return Err(AuthenticationError::Unauthorized);
        }
        Ok(())
    }
}

use std::borrow::Cow;
use std::fmt::{Debug, Display};
use std::ops::Deref;

use axum::extract::{FromRef, FromRequestParts, OptionalFromRequestParts};
use axum::response::IntoResponse;
use axum_extra::extract::cookie::Cookie;
use derive_more::From;

use http::request::Parts;
use nr_core::database::DBError;
use nr_core::database::entities::user::auth_token::AuthToken;
use nr_core::database::entities::user::{UserModel, UserSafeData, UserType};
use nr_core::user::permissions::{HasPermissions, UserPermissions};
use serde::Serialize;
use session::{Session, SessionError};
use sqlx::PgPool;
use strum::EnumIs;
use thiserror::Error;
use tracing::{error, instrument, warn};
use utoipa::ToSchema;

use crate::error::IntoErrorResponse;
use crate::utils::headers::AuthorizationHeader;
use crate::utils::response_builder::ResponseBuilder;
use crate::utils::responses::APIErrorResponse;

use super::NitroRepo;

pub mod layer;
pub mod session;
pub mod ws;

#[derive(Error, Debug)]
pub enum AuthenticationError {
    #[error("Internal Authentication Error: {0}")]
    InternalError(Box<dyn IntoErrorResponse>),
    #[error("Unable to Authenticate")]
    Unauthorized,
    #[error("Password is not able to be verified.")]
    PasswordVerificationError,
    #[error("No Auth Token Allowed here")]
    AuthTokenForbidden,
    #[error("Forbidden")]
    Forbidden,
}
impl AuthenticationError {
    pub fn is_internal_error(&self) -> bool {
        matches!(self, AuthenticationError::InternalError(_))
    }
}
macro_rules! internal_errors {
    (
        $($error:ty),*
    ) => {
        $(
            impl From<$error> for AuthenticationError {
                fn from(err: $error) -> Self {
                    AuthenticationError::InternalError(Box::new(err))
                }
            }
        )*
    };
}
internal_errors!(SessionError, sqlx::Error, DBError);
impl IntoErrorResponse for AuthenticationError {
    fn into_response_boxed(self: Box<Self>) -> axum::response::Response {
        self.into_response()
    }

    fn status_code(&self) -> http::StatusCode {
        match self {
            AuthenticationError::InternalError(_) => http::StatusCode::INTERNAL_SERVER_ERROR,
            AuthenticationError::Unauthorized => http::StatusCode::UNAUTHORIZED,
            AuthenticationError::PasswordVerificationError => http::StatusCode::UNAUTHORIZED,
            AuthenticationError::AuthTokenForbidden => http::StatusCode::FORBIDDEN,
            AuthenticationError::Forbidden => http::StatusCode::FORBIDDEN,
        }
    }
}
impl IntoResponse for AuthenticationError {
    fn into_response(self) -> axum::response::Response {
        error!("Authentication Error: {}", self);
        match self {
            AuthenticationError::InternalError(err) => err.into_response_boxed(),
            AuthenticationError::AuthTokenForbidden => {
                let api_error = APIErrorResponse::<(), ()> {
                    message: Cow::Borrowed(
                        "This route Forbids Auth Tokens as a form of Authentication",
                    ),
                    details: None,
                    error: None,
                };
                ResponseBuilder::forbidden().json(&api_error)
            }
            other => {
                let status_code = other.status_code();
                let api_error = APIErrorResponse::<(), AuthenticationError> {
                    message: Cow::Borrowed("Authentication Error"),
                    details: None,
                    error: Some(other),
                };
                ResponseBuilder::default()
                    .status(status_code)
                    .json(&api_error)
            }
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
impl<S> OptionalFromRequestParts<S> for Authentication
where
    NitroRepo: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = AuthenticationError;
    async fn from_request_parts(
        parts: &mut Parts,
        state: &S,
    ) -> Result<Option<Self>, Self::Rejection> {
        let raw_extension = parts.extensions.get::<AuthenticationRaw>().cloned();
        let repo = NitroRepo::from_ref(state);
        let Some(raw_auth) = raw_extension else {
            return Ok(None);
        };
        let auth = match raw_auth {
            AuthenticationRaw::NoIdentification => {
                return Ok(None);
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
                return Ok(None);
            }
        };
        Ok(Some(auth))
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

impl AuthenticationRaw {
    pub fn method_name(&self) -> Option<&str> {
        match self {
            AuthenticationRaw::AuthToken(_) => Some("Auth Token"),
            AuthenticationRaw::Session(_) => Some("Session"),
            AuthenticationRaw::AuthorizationHeaderUnknown(_, _) => {
                Some("Authorization Header Unknown")
            }
            AuthenticationRaw::Basic { .. } => Some("Basic"),
            AuthenticationRaw::NoIdentification => None,
        }
    }
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
        Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
        password_hash::{Salt, SaltString},
    };
    use rand::{TryRngCore, rngs::OsRng};
    use tracing::{error, instrument};

    use crate::app::authentication::AuthenticationError;
    #[instrument(skip(password), fields(project_module = "Authentication"))]
    pub fn encrypt_password(password: &str) -> Option<String> {
        let mut bytes = [0u8; Salt::RECOMMENDED_LENGTH];
        OsRng
            .try_fill_bytes(&mut bytes)
            .expect("Failed to generate random bytes");
        let salt = SaltString::encode_b64(&bytes).expect("Failed to generate salt");
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

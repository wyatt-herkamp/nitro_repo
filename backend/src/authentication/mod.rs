use std::{
    fmt,
    fmt::{Debug, Display, Formatter},
};

use actix_web::{
    dev::Payload, http::StatusCode, web::Data, FromRequest, HttpMessage, HttpRequest, ResponseError,
};
use argon2::{Argon2, PasswordHash, PasswordVerifier};
use futures_util::future::LocalBoxFuture;
use log::warn;
use sea_orm::{ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter};
use serde::Deserialize;
use serde_json::json;
use this_actix_error::ActixError;
use thiserror::Error;

use crate::{
    authentication::{auth_token::AuthTokenModel, session::Session},
    error::internal_error::InternalError,
    system::user::{
        database::{Column::Username, UserSafeData},
        UserEntity, UserModel,
    },
};
#[derive(Debug, Error, ActixError)]
pub enum AuthenticationError {
    #[status_code(UNAUTHORIZED)]
    #[error("Unauthorized")]
    NoAuthenticationProvided,
    #[status_code(UNAUTHORIZED)]
    #[error("Invalid API Key")]
    InvalidAPIKey,
    #[status_code(UNAUTHORIZED)]
    #[error("Invalid Session")]
    InvalidSession,
    #[status_code(FORBIDDEN)]
    #[error("Must be a session")]
    MustBeSession,
    #[status_code(BAD_REQUEST)]
    #[error("Invalid Basic Auth Header")]
    InvalidFormatBasicAuth,
    #[status_code(UNAUTHORIZED)]
    #[error("Invalid Login")]
    InvalidLogin,
    #[error("Database Error")]
    #[status_code(INTERNAL_SERVER_ERROR)]
    DatabaseError(#[from] DbErr),
    #[error("Internal Error")]
    #[status_code(INTERNAL_SERVER_ERROR)]
    InternalError(#[from] InternalError),
    #[error("No Authentication Provided")]
    #[status_code(UNAUTHORIZED)]
    NoAuthentication,
    #[status_code(BAD_REQUEST)]
    #[error("Invalid Authentication Format: {0}")]
    InvalidAuthenticationFormat(String),
}

pub mod auth_token;
pub mod middleware;
pub mod session;

pub struct NotAuthenticated;

impl Debug for NotAuthenticated {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Not Authenticated")
    }
}

impl Display for NotAuthenticated {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let result = serde_json::to_string(&json!({
            "error": "Not Authenticated",
        }))
        .map_err(|_| fmt::Error)?;
        write!(f, "{}", result)
    }
}

impl ResponseError for NotAuthenticated {
    fn status_code(&self) -> StatusCode {
        StatusCode::UNAUTHORIZED
    }
}
#[derive(Clone, Debug, PartialEq)]
pub enum RawAuthentication {
    AuthToken(String),
    Session(Session),
    Basic(String),
    AuthorizationHeaderUnknown(String, String),
}
/// Could be any type of authentication
///
/// Could be an unknown type of authentication. Not Secure to use use unless you call `authorized()` or get_user()`
#[derive(Clone, Debug, PartialEq)]
pub enum Authentication {
    /// An Auth Token was passed under the Authorization Header
    AuthToken(AuthTokenModel, UserSafeData),
    /// Session Value from Cookie
    Session(Session, UserSafeData),
    /// If the Authorization Header could not be parsed. Give them the value
    AuthorizationHeaderUnknown(String, String),
    /// Authorization Basic Header
    Basic(UserSafeData),
    /// No Authentication was provided
    /// Could be a valid request, but not authenticated
    NoAuthentication,
}

/// Is a known authentication type
#[derive(Clone, Debug, PartialEq)]
pub enum TrulyAuthenticated {
    /// An Auth Token was passed under the Authorization Header
    AuthToken(AuthTokenModel, UserSafeData),
    /// Session Value from Cookie
    Session(Session, UserSafeData),
    /// Authorization Basic Header
    Basic(UserSafeData),
}
impl TryFrom<Authentication> for TrulyAuthenticated {
    type Error = AuthenticationError;

    fn try_from(value: Authentication) -> Result<Self, Self::Error> {
        match value {
            Authentication::AuthToken(token, user) => {
                Ok(TrulyAuthenticated::AuthToken(token, user))
            }
            Authentication::Session(session, user) => {
                Ok(TrulyAuthenticated::Session(session, user))
            }
            Authentication::Basic(user) => Ok(TrulyAuthenticated::Basic(user)),
            Authentication::AuthorizationHeaderUnknown(auth_type, _) => {
                warn!("Unknown Auth Type: {}", auth_type);
                Err(AuthenticationError::InvalidAuthenticationFormat(auth_type))
            }
            Authentication::NoAuthentication => Err(AuthenticationError::NoAuthentication),
        }
    }
}
impl TrulyAuthenticated {
    pub fn into_authenticated(self) -> Authentication {
        match self {
            TrulyAuthenticated::AuthToken(token, user) => Authentication::AuthToken(token, user),
            TrulyAuthenticated::Session(session, user) => Authentication::Session(session, user),
            TrulyAuthenticated::Basic(user) => Authentication::Basic(user),
        }
    }

    pub fn into_user(self) -> UserSafeData {
        match self {
            TrulyAuthenticated::AuthToken(_, user) => user,
            TrulyAuthenticated::Session(_, user) => user,
            TrulyAuthenticated::Basic(user) => user,
        }
    }
    pub async fn new(
        raw: RawAuthentication,
        database: Data<DatabaseConnection>,
    ) -> Result<Self, AuthenticationError> {
        let authentication = Authentication::new(raw, database).await?;
        let authentication = authentication.try_into()?;
        Ok(authentication)
    }
}
impl Authentication {
    pub fn authorized(&self) -> bool {
        !matches!(self, Authentication::AuthorizationHeaderUnknown(_, _))
    }
    pub async fn new(
        raw: RawAuthentication,
        database: Data<DatabaseConnection>,
    ) -> Result<Self, AuthenticationError> {
        match raw {
            RawAuthentication::AuthToken(token) => {
                let token = auth_token::get_token(token, &database).await?;
                let (token, user) = token.ok_or(AuthenticationError::InvalidAPIKey)?;
                Ok(Authentication::AuthToken(token, user))
            }
            RawAuthentication::Session(session) => {
                let user = UserEntity::find_by_id(session.user_id)
                    .one(database.as_ref())
                    .await?
                    .ok_or(AuthenticationError::InvalidSession)?;
                Ok(Authentication::Session(session, user.into()))
            }
            RawAuthentication::Basic(basic) => {
                let basic = utils::decode_base64_as_string(basic)?;
                let split: Vec<&str> = basic.split(":").collect();
                if split.len() != 2 {
                    return Err(AuthenticationError::InvalidFormatBasicAuth);
                }
                let username = split[0];
                let password = split[1];
                verify_login(username, password, &database)
                    .await?
                    .map(Authentication::Basic)
                    .ok_or(AuthenticationError::InvalidLogin)
            }
            RawAuthentication::AuthorizationHeaderUnknown(auth_type, value) => {
                Ok(Authentication::AuthorizationHeaderUnknown(auth_type, value))
            }
        }
    }
    pub fn get_user(self) -> Option<UserSafeData> {
        match self {
            Authentication::AuthToken(_, user) => Some(user),
            Authentication::Session(_, user) => Some(user),
            Authentication::Basic(user) => Some(user),
            _ => None,
        }
    }
}
impl FromRequest for Authentication {
    type Error = AuthenticationError;
    type Future = LocalBoxFuture<'static, Result<Authentication, Self::Error>>;

    #[inline]
    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        let model = req.extensions_mut().get::<RawAuthentication>().cloned();
        let database = req.app_data::<Data<DatabaseConnection>>().unwrap().clone();
        if let Some(model) = model {
            Box::pin(async move { Authentication::new(model, database).await })
        } else {
            Box::pin(async move { Ok(Authentication::NoAuthentication) })
        }
    }
}
impl FromRequest for TrulyAuthenticated {
    type Error = AuthenticationError;
    type Future = LocalBoxFuture<'static, Result<TrulyAuthenticated, Self::Error>>;

    #[inline]
    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        let model = req.extensions_mut().get::<RawAuthentication>().cloned();
        let database = req.app_data::<Data<DatabaseConnection>>().unwrap().clone();
        if let Some(model) = model {
            Box::pin(async move { TrulyAuthenticated::new(model, database).await })
        } else {
            Box::pin(async move { Err(AuthenticationError::NoAuthenticationProvided) })
        }
    }
}

#[derive(Deserialize, Clone, Debug)]
pub struct SecureAction<T: Clone + Debug> {
    pub username: String,
    pub password: String,
    pub secure_data: T,
}
impl<T: Clone + Debug> SecureAction<T> {
    pub async fn verify(
        &self,
        database: &DatabaseConnection,
    ) -> Result<Option<UserSafeData>, AuthenticationError> {
        let user = verify_login(&self.username, &self.password, database)
            .await?
            .ok_or(AuthenticationError::InvalidLogin)?;
        Ok(Some(user))
    }
    pub fn into_inner(self) -> T {
        self.secure_data
    }
}
#[inline(always)]
pub async fn verify_login(
    username: impl AsRef<str>,
    password: impl AsRef<str>,
    database: &DatabaseConnection,
) -> Result<Option<UserSafeData>, AuthenticationError> {
    let user_found: Option<UserModel> = UserEntity::find()
        .filter(Username.eq(username.as_ref()))
        .one(database)
        .await?;
    if user_found.is_none() {
        return Ok(None);
    }
    let argon2 = Argon2::default();
    let user = user_found.unwrap();
    let parsed_hash = PasswordHash::new(user.password.as_str()).map_err(|v| {
        warn!("Error Parsing Password Hash: {:?}", v);
        AuthenticationError::InternalError(InternalError::from(v))
    })?;
    if argon2
        .verify_password(password.as_ref().as_bytes(), &parsed_hash)
        .is_err()
    {
        return Ok(None);
    }
    Ok(Some(user.into()))
}
mod utils {
    use base64::{engine::general_purpose::STANDARD, DecodeError, Engine};

    use super::AuthenticationError;

    pub fn decode_base64(input: impl AsRef<[u8]>) -> Result<Vec<u8>, DecodeError> {
        STANDARD.decode(input)
    }
    pub fn decode_base64_as_string(input: impl AsRef<[u8]>) -> Result<String, AuthenticationError> {
        let decoded =
            decode_base64(input).map_err(|_| AuthenticationError::InvalidFormatBasicAuth)?;
        String::from_utf8(decoded).map_err(|_| AuthenticationError::InvalidFormatBasicAuth)
    }
}

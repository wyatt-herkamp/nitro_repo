use std::fmt;
use std::fmt::{Debug, Display, Formatter};

use actix_web::dev::Payload;
use actix_web::http::StatusCode;
use actix_web::{FromRequest, HttpMessage, HttpRequest, ResponseError};
use argon2::{Argon2, PasswordHash, PasswordVerifier};
use futures_util::future::{ready, Ready};
use nr_entities::user::api_token::AuthTokenModel;
use nr_entities::user::{UserColumn, UserEntity, UserModel, UserSafeData};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use serde::Deserialize;
use serde_json::json;
use session::Session;
use tracing::trace;

use crate::error::internal_error::InternalError;

pub mod api_middleware;
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
pub enum Authentication {
    /// Neither a Session or Auth Token exist.
    /// Might deny these requests in the future on API routes
    NoIdentification,
    /// An Auth Token was passed under the Authorization Header
    AuthToken(AuthTokenModel, UserSafeData),
    /// Session Value from Cookie
    Session(Session),
    /// If the Authorization Header could not be parsed. Give them the value
    AuthorizationHeaderUnknown(String, String),
    /// Authorization Basic Header
    Basic(UserSafeData),
}

impl Authentication {
    pub fn authorized(&self) -> bool {
        if let Authentication::NoIdentification = &self {
            return false;
        }
        if let Authentication::Session(session) = &self {
            return true;
        }
        true
    }
    pub async fn get_user(
        self,
        database: &DatabaseConnection,
    ) -> Result<Result<UserSafeData, NotAuthenticated>, InternalError> {
        match self {
            Authentication::AuthToken(_, user) => Ok(Ok(user)),
            Authentication::Session(session) => {
                if let Some(user) = Some(session.user_id) {
                    let option = UserEntity::find_by_id(user)
                        .into_model()
                        .one(database)
                        .await?;
                    if let Some(user) = option {
                        Ok(Ok(user))
                    } else {
                        Ok(Err(NotAuthenticated))
                    }
                } else {
                    Ok(Err(NotAuthenticated))
                }
            }
            Authentication::Basic(user) => Ok(Ok(user)),
            _ => Ok(Err(NotAuthenticated)),
        }
    }
}

impl FromRequest for Authentication {
    type Error = InternalError;
    type Future = Ready<Result<Authentication, Self::Error>>;

    #[inline]
    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        let model = req.extensions_mut().get::<Authentication>().cloned();
        if model.is_none() {
            trace!("Missing Extension");
            return ready(Ok(Authentication::NoIdentification));
        }

        ready(Ok(model.unwrap()))
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
    ) -> Result<Result<UserSafeData, NotAuthenticated>, InternalError> {
        verify_login(&self.username, &self.password, database).await
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
) -> Result<Result<UserSafeData, NotAuthenticated>, InternalError> {
    let user_found: Option<UserModel> = UserEntity::find()
        .filter(UserColumn::Username.eq(username.as_ref()))
        .one(database)
        .await?;
    if user_found.is_none() {
        return Ok(Err(NotAuthenticated));
    }
    let argon2 = Argon2::default();
    let user = user_found.unwrap();
    let parsed_hash = PasswordHash::new(user.password.as_str())?;
    if argon2
        .verify_password(password.as_ref().as_bytes(), &parsed_hash)
        .is_err()
    {
        return Ok(Err(NotAuthenticated));
    }
    Ok(Ok(user.into()))
}

#[derive(Debug, Clone)]
pub enum AuthenticationRaw {
    /// Neither a Session or Auth Token exist.
    /// Might deny these requests in the future on API routes
    NoIdentification,
    /// An Auth Token was passed under the Authorization Header
    AuthToken(AuthTokenModel, UserSafeData),
    /// Session Value from Cookie
    Session(Session),
    /// If the Authorization Header could not be parsed. Give them the value
    AuthorizationHeaderUnknown(String, String),
    /// Authorization Basic Header
    Basic(UserSafeData),
}

pub mod auth_token;
pub mod middleware;
pub mod session;

use actix_web::body::BoxBody;
use actix_web::dev::Payload;
use actix_web::http::StatusCode;
use actix_web::{FromRequest, HttpMessage, HttpRequest, HttpResponse, ResponseError};
use argon2::{Argon2, PasswordHash, PasswordVerifier};
use std::fmt::{Debug, Display, Formatter};

use futures_util::future::{ready, Ready};
use log::trace;

use sea_orm::{DatabaseConnection, EntityTrait};

use crate::authentication::auth_token::AuthTokenModel;
use crate::authentication::session::Session;
use crate::error::internal_error::InternalError;
use crate::system::user;

use crate::system::user::{UserEntity, UserModel};
pub struct NotAuthenticated;

impl Debug for NotAuthenticated {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Not Authenticated")
    }
}

impl Display for NotAuthenticated {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Not Authenticated")
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
    AuthToken(AuthTokenModel),
    /// Session Value from Cookie
    Session(Session),
    /// If the Authorization Header could not be parsed. Give them the value
    AuthorizationHeaderUnknown(String, String),
    /// Authorization Basic Header
    Basic(UserModel),
}

impl Authentication {
    pub fn authorized(&self) -> bool {
        if let Authentication::NoIdentification = &self {
            return false;
        }
        if let Authentication::Session(session) = &self {
            return session.user.is_some();
        }
        true
    }
    pub async fn get_user(
        self,
        database: &DatabaseConnection,
    ) -> Result<Result<UserModel, NotAuthenticated>, InternalError> {
        match self {
            Authentication::AuthToken(auth) => {
                let option = auth.get_user(database).await?;
                if let Some(user) = option {
                    Ok(Ok(user))
                } else {
                    Ok(Err(NotAuthenticated))
                }
            }
            Authentication::Session(session) => {
                if let Some(user) = session.user {
                    let option = UserEntity::find_by_id(user).one(database).await?;
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

pub async fn verify_login(
    username: String,
    password: String,
    database: &DatabaseConnection,
) -> Result<Result<UserModel, NotAuthenticated>, InternalError> {
    let user_found: Option<UserModel> = user::get_by_username(&username, database).await?;
    if user_found.is_none() {
        return Ok(Err(NotAuthenticated));
    }
    let argon2 = Argon2::default();
    let user = user_found.unwrap();
    let parsed_hash = PasswordHash::new(user.password.as_str())?;
    if argon2
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_err()
    {
        return Ok(Err(NotAuthenticated));
    }
    Ok(Ok(user))
}

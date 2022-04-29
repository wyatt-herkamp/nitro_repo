pub mod basic;
pub mod middleware;

use std::fmt::{Debug, Display, Formatter};
use crate::session::basic::BasicSessionManager;
use actix_web::dev::Payload;
use actix_web::{FromRequest, HttpMessage, HttpRequest, HttpResponse, ResponseError};
use actix_web::body::BoxBody;
use actix_web::http::StatusCode;
use async_trait::async_trait;
use futures_util::future::{ready, Ready};
use log::trace;


use sea_orm::DatabaseConnection;
use time::OffsetDateTime;

use crate::error::internal_error::InternalError;
use crate::settings::models::SessionSettings;
use crate::{APIResponse, system};
use crate::system::user::Model as User;
use system::auth_token::Model as AuthToken;
use crate::api_response::RequestErrorResponse;

pub struct UnAuthorized;

impl Debug for UnAuthorized {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Request was unauthorized")
    }
}

impl Display for UnAuthorized {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Request was unauthorized")
    }
}

impl std::error::Error for UnAuthorized {}

impl ResponseError for UnAuthorized {
    fn status_code(&self) -> StatusCode {
        StatusCode::UNAUTHORIZED
    }
    fn error_response(&self) -> HttpResponse<BoxBody> {
        HttpResponse::Ok()
            .status(StatusCode::UNAUTHORIZED)
            .content_type("application/json")
            .body(serde_json::to_string(&APIResponse {
                success: false,
                data: Some(RequestErrorResponse::new("Not Logged In", "UNAUTHORIZED")),
                status_code: Some(401),
            }).unwrap())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Authentication {
    /// Neither a Session or Auth Token exist.
    /// Might deny these requests in the future on API routes
    NoIdentification,
    /// An Auth Token was passed under the Authorization Header
    AuthToken(AuthToken),
    /// Session Value from Cookie
    Session(Session),
    /// If the Authorization Header could not be parsed. Give them the value
    AuthorizationHeaderUnkown(String, String),
    /// Authorization Basic Header
    Basic(User),
}

impl Authentication {
    pub fn authorized(&self) -> bool {
        if let Authentication::NoIdentification = &self {
            return false;
        }
        if let Authentication::Session(session) = &self {
            return session.auth_token.is_some();
        }
        true
    }
    pub fn get_auth_token(self) -> Result<AuthToken, UnAuthorized> {
        match self {
            Authentication::AuthToken(auth) => Ok(auth),
            Authentication::Session(session) => {
                if let Some(auth) = session.auth_token {
                    Ok(auth)
                } else {
                    Err(UnAuthorized)
                }
            }

            _ => Err(UnAuthorized),
        }
    }
    pub async fn get_user(
        self,
        database: &DatabaseConnection,
    ) -> Result<Result<User, UnAuthorized>, InternalError> {
        match self {
            Authentication::AuthToken(auth) => {
                let option = auth
                    .get_user(database)
                    .await?;
                if let Some(user) = option {
                    Ok(Ok(user))
                } else {
                    Ok(Err(UnAuthorized))
                }
            }
            Authentication::Session(session) => {
                if let Some(auth_token) = session.auth_token {
                    let option = auth_token
                        .get_user(database)
                        .await?;
                    if let Some(user) = option {
                        Ok(Ok(user))
                    } else {
                        Ok(Err(UnAuthorized))
                    }
                } else {
                    Ok(Err(UnAuthorized))
                }
            }
            Authentication::Basic(user) => Ok(Ok(user)),
            _ => Ok(Err(UnAuthorized)),
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

pub enum SessionManager {
    BasicSessionManager(BasicSessionManager),
}

#[derive(Clone, Debug, PartialEq)]
pub struct Session {
    pub token: String,
    pub auth_token: Option<AuthToken>,
    pub expiration: OffsetDateTime,
}

#[async_trait]
pub trait SessionManagerType {
    type Error;
    async fn delete_session(&self, token: &str) -> Result<(), Self::Error>;
    async fn create_session(&self) -> Result<Session, Self::Error>;
    async fn retrieve_session(&self, token: &str) -> Result<Option<Session>, Self::Error>;
    async fn re_create_session(&self, token: &str) -> Result<Session, Self::Error>;
    async fn set_auth_token(&self, token: &str, auth_token: AuthToken) -> Result<(), Self::Error>;
}

#[async_trait]
impl SessionManagerType for SessionManager {
    type Error = ();

    async fn delete_session(&self, token: &str) -> Result<(), Self::Error> {
        return match self {
            SessionManager::BasicSessionManager(basic) => basic.delete_session(token).await,
        };
    }

    async fn create_session(&self) -> Result<Session, Self::Error> {
        return match self {
            SessionManager::BasicSessionManager(basic) => basic.create_session().await,
        };
    }

    async fn retrieve_session(&self, token: &str) -> Result<Option<Session>, Self::Error> {
        return match self {
            SessionManager::BasicSessionManager(basic) => basic.retrieve_session(token).await,
        };
    }

    async fn re_create_session(&self, token: &str) -> Result<Session, Self::Error> {
        return match self {
            SessionManager::BasicSessionManager(basic) => basic.re_create_session(token).await,
        };
    }

    async fn set_auth_token(&self, token: &str, auth_token: AuthToken) -> Result<(), Self::Error> {
        return match self {
            SessionManager::BasicSessionManager(basic) => {
                basic.set_auth_token(token, auth_token).await
            }
        };
    }
}

impl TryFrom<SessionSettings> for SessionManager {
    type Error = String;

    fn try_from(value: SessionSettings) -> Result<Self, Self::Error> {
        return match value.manager.as_str() {
            "BasicSessionManager" => Ok(SessionManager::BasicSessionManager(
                BasicSessionManager::default(),
            )),
            _ => Err("Invalid Session Manager".to_string()),
        };
    }
}

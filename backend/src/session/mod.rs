pub mod basic;
pub mod middleware;

use actix_web::{FromRequest, HttpMessage, HttpRequest};
use actix_web::dev::Payload;
use async_trait::async_trait;
use futures_util::future::{Ready, ready};
use lettre::transport::smtp::commands::Auth;
use sea_orm::DatabaseConnection;
use sea_orm::sea_query::ColumnSpec::Default;
use time::OffsetDateTime;
use crate::session::basic::BasicSessionManager;

use crate::system;
use system::auth_token::Model as AuthToken;
use crate::error::internal_error::InternalError;
use crate::settings::models::SessionSettings;
use crate::system::user::Model as User;

#[derive(Clone, Debug, PartialEq)]
pub enum Authentication {
    /// Neither a Session or Auth Token exist.
    /// Might deny these requests in the future on API routes
    None,
    /// An Auth Token was passed under the Authorization Header
    AuthToken(AuthToken),
    /// Session Value from Cookie
    Session(Session),
    /// Authorization Basic Header
    Basic(User),
}

impl Authentication {
    pub fn authorized(&self) -> bool {
        if let Authentication::None = &self {
            return false;
        }
        if let Authentication::Session(session) = &self {
            return session.auth_token.is_some();
        }
        return true;
    }
    pub fn get_auth_token(self) -> Option<AuthToken> {
        match self {
            Authentication::AuthToken(auth) => {
                Some(auth)
            }
            Authentication::Session(session) => {
                session.auth_token
            }

            _ => { None }
        }
    }
    pub async fn get_user(self, database: &DatabaseConnection) -> Result<Option<User>, InternalError> {
        match self {
            Authentication::AuthToken(auth) => {
                auth.get_user(database).await.map_err(InternalError::DBError)
            }
            Authentication::Session(session) => {
                if let Some(auth_token) = session.auth_token {
                    auth_token.get_user(database).await.map_err(InternalError::DBError)
                } else {
                    Ok(None)
                }
            }
            Authentication::Basic(user) => {
                Ok(Some(user))
            }
            _ => { Ok(None) }
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
            return ready(Ok(Authentication::None));
        }

        ready(Ok(model.unwrap()))
    }
}


pub enum SessionManager {
    BasicSessionManager(BasicSessionManager)
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
        return match self { SessionManager::BasicSessionManager(basic) => { basic.delete_session(token).await } };
    }

    async fn create_session(&self) -> Result<Session, Self::Error> {
        return match self { SessionManager::BasicSessionManager(basic) => { basic.create_session().await } };
    }

    async fn retrieve_session(&self, token: &str) -> Result<Option<Session>, Self::Error> {
        return match self { SessionManager::BasicSessionManager(basic) => { basic.retrieve_session(token).await } };
    }

    async fn re_create_session(&self, token: &str) -> Result<Session, Self::Error> {
        return match self { SessionManager::BasicSessionManager(basic) => { basic.re_create_session(token).await } };
    }

    async fn set_auth_token(&self, token: &str, auth_token: AuthToken) -> Result<(), Self::Error> {
        return match self { SessionManager::BasicSessionManager(basic) => { basic.set_auth_token(token, auth_token).await } };
    }
}

impl TryFrom<SessionSettings> for SessionManager {
    type Error = String;

    fn try_from(value: SessionSettings) -> Result<Self, Self::Error> {
        return match value.manager.as_str() {
            "BasicSessionManager" => {
                Ok(SessionManager::BasicSessionManager(BasicSessionManager::default()))
            }
            _ => {
                Err("Invalid Session Manager".to_string())
            }
        };
    }
}
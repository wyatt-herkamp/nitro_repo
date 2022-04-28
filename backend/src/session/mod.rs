pub mod basic;
pub mod middleware;

use async_trait::async_trait;
use sea_orm::sea_query::ColumnSpec::Default;
use time::OffsetDateTime;
use crate::session::basic::BasicSessionManager;

use crate::system;
use system::auth_token::Model as AuthToken;
use crate::settings::models::SessionSettings;

pub enum SessionManager {
    BasicSessionManager(BasicSessionManager)
}

#[derive(Clone, Debug)]
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
        return  match self { SessionManager::BasicSessionManager(basic) => { basic.create_session().await } };
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
pub mod basic;
pub mod redis_manager;

use crate::authentication::session::basic::BasicSessionManager;
use crate::authentication::session::redis_manager::RedisSessionManager;
use crate::settings::models::SessionSettings;
use async_trait::async_trait;
use thiserror::Error;

pub enum SessionManager {
    BasicSessionManager(BasicSessionManager),
    RedisSessionManager(RedisSessionManager),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Session {
    pub token: String,
    pub user: Option<i64>,
    pub expiration: u64,
}
#[derive(Error, Debug)]
pub enum SessionError {
    #[error("As of Now. This can not happen")]
    BasicError,
    #[error("Error with the Redis Session Manager. {0}")]
    RedisError(redis::RedisError),
}
#[async_trait]
pub trait SessionManagerType {
    type Error;
    async fn delete_session(&self, token: &str) -> Result<(), Self::Error>;
    async fn create_session(&self) -> Result<Session, Self::Error>;
    async fn retrieve_session(&self, token: &str) -> Result<Option<Session>, Self::Error>;
    async fn re_create_session(&self, token: &str) -> Result<Session, Self::Error>;
    async fn set_user(&self, token: &str, user: i64) -> Result<(), Self::Error>;
}

#[async_trait]
impl SessionManagerType for SessionManager {
    type Error = SessionError;

    async fn delete_session(&self, token: &str) -> Result<(), Self::Error> {
        return match self {
            SessionManager::BasicSessionManager(basic) => basic
                .delete_session(token)
                .await
                .map_err(|_| SessionError::BasicError),
            SessionManager::RedisSessionManager(basic) => basic
                .delete_session(token)
                .await
                .map_err(SessionError::RedisError),
        };
    }

    async fn create_session(&self) -> Result<Session, Self::Error> {
        return match self {
            SessionManager::BasicSessionManager(basic) => basic
                .create_session()
                .await
                .map_err(|_| SessionError::BasicError),
            SessionManager::RedisSessionManager(basic) => basic
                .create_session()
                .await
                .map_err(SessionError::RedisError),
        };
    }

    async fn retrieve_session(&self, token: &str) -> Result<Option<Session>, Self::Error> {
        return match self {
            SessionManager::BasicSessionManager(basic) => basic
                .retrieve_session(token)
                .await
                .map_err(|_| SessionError::BasicError),

            SessionManager::RedisSessionManager(basic) => basic
                .retrieve_session(token)
                .await
                .map_err(SessionError::RedisError),
        };
    }

    async fn re_create_session(&self, token: &str) -> Result<Session, Self::Error> {
        return match self {
            SessionManager::BasicSessionManager(basic) => basic
                .re_create_session(token)
                .await
                .map_err(|_| SessionError::BasicError),

            SessionManager::RedisSessionManager(basic) => basic
                .re_create_session(token)
                .await
                .map_err(SessionError::RedisError),
        };
    }

    async fn set_user(&self, token: &str, user: i64) -> Result<(), Self::Error> {
        return match self {
            SessionManager::BasicSessionManager(basic) => basic
                .set_user(token, user)
                .await
                .map_err(|_| SessionError::BasicError),
            SessionManager::RedisSessionManager(basic) => basic
                .set_user(token, user)
                .await
                .map_err(SessionError::RedisError),
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

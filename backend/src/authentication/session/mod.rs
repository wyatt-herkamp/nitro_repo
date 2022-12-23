pub mod basic;

use crate::authentication::session::basic::BasicSessionManager;
use crate::settings::models::SessionSettings;
use async_trait::async_trait;
use chrono::{DateTime, Duration, Local};
use log::error;
use std::borrow::Cow;
use std::sync::Arc;
use thiserror::Error;

pub enum SessionManager {
    BasicSessionManager(BasicSessionManager),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Session {
    pub token: String,
    pub user: Option<i64>,
    pub expiration: DateTime<Local>,
}

#[derive(Error, Debug)]
pub enum SessionError {
    #[error("As of Now. This can not happen")]
    BasicError,
}

#[async_trait]
pub trait SessionManagerType {
    type Error;
    async fn delete_session(&self, token: &str) -> Result<(), Self::Error>;
    async fn create_session(&self) -> Result<Session, Self::Error>;
    async fn retrieve_session(&self, token: &str) -> Result<Option<Session>, Self::Error>;
    async fn re_create_session(&self, token: &str) -> Result<Session, Self::Error>;
    async fn set_user(&self, token: &str, user: i64) -> Result<(), Self::Error>;

    async fn push_session(&self, session: Session) -> Result<(), Self::Error>;

    async fn clean_sessions(&self) -> Result<(), Self::Error>;
}
pub async fn session_cleaner(session_data: Arc<SessionManager>) {
    let mut interval =
        tokio::time::interval(Duration::hours(1).to_std().expect("Invalid Duration"));
    loop {
        interval.tick().await;
        if let Err(e) = session_data.clean_sessions().await {
            error!("Error cleaning sessions: {:?}", e);
        }
    }
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
        };
    }

    async fn create_session(&self) -> Result<Session, Self::Error> {
        return match self {
            SessionManager::BasicSessionManager(basic) => basic
                .create_session()
                .await
                .map_err(|_| SessionError::BasicError),
        };
    }

    async fn retrieve_session(&self, token: &str) -> Result<Option<Session>, Self::Error> {
        return match self {
            SessionManager::BasicSessionManager(basic) => basic
                .retrieve_session(token)
                .await
                .map_err(|_| SessionError::BasicError),
        };
    }

    async fn re_create_session(&self, token: &str) -> Result<Session, Self::Error> {
        return match self {
            SessionManager::BasicSessionManager(basic) => basic
                .re_create_session(token)
                .await
                .map_err(|_| SessionError::BasicError),
        };
    }

    async fn set_user(&self, token: &str, user: i64) -> Result<(), Self::Error> {
        return match self {
            SessionManager::BasicSessionManager(basic) => basic
                .set_user(token, user)
                .await
                .map_err(|_| SessionError::BasicError),
        };
    }

    async fn push_session(&self, session: Session) -> Result<(), Self::Error> {
        return match self {
            SessionManager::BasicSessionManager(basic) => basic
                .push_session(session)
                .await
                .map_err(|_| SessionError::BasicError),
        };
    }

    async fn clean_sessions(&self) -> Result<(), Self::Error> {
        return match self {
            SessionManager::BasicSessionManager(basic) => basic
                .clean_sessions()
                .await
                .map_err(|_| SessionError::BasicError),
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

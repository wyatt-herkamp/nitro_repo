pub mod basic;

use async_trait::async_trait;
use sea_orm::sea_query::ColumnSpec::Default;
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
    pub expiration: u128,
}

#[async_trait]
pub trait SessionManagerType {
    type Error;
    async fn delete_session(&self, token: &str) -> Result<(), Self::Error>;
    async fn create_session(&self) -> Result<Session, Self::Error>;
    async fn retrieve_session(&self, token: &str) -> Result<Option<Session>, Self::Error>;
    async fn set_auth_token(&self, token: &str, auth_token: AuthToken) -> Result<(), Self::Error>;
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
        }
    }
}
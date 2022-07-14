use crate::authentication::session::{Session, SessionManager, SessionManagerType};
use async_trait::async_trait;
use redis::{Commands, RedisResult};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RedisConfig {
    /// The URL for the Redis server.
    /// Please follow https://docs.rs/redis/latest/redis/#connection-parameters
    pub url: String,
}
pub struct RedisSessionManager {
    pub client: RwLock<redis::Client>,
}

impl RedisSessionManager {
    pub fn new(redis: RedisConfig) -> RedisResult<RedisSessionManager> {
        let client = redis::Client::open(redis.url.as_str())?;
        Ok(RedisSessionManager {
            client: RwLock::new(client),
        })
    }
}
#[async_trait]
impl SessionManagerType for RedisSessionManager {
    type Error = redis::RedisError;

    async fn delete_session(&self, token: &str) -> Result<(), Self::Error> {
        todo!()
    }

    async fn create_session(&self) -> Result<Session, Self::Error> {
        todo!()
    }

    async fn retrieve_session(&self, token: &str) -> Result<Option<Session>, Self::Error> {
        todo!()
    }

    async fn re_create_session(&self, token: &str) -> Result<Session, Self::Error> {
        todo!()
    }

    async fn set_user(&self, token: &str, user: i64) -> Result<(), Self::Error> {
        todo!()
    }
}

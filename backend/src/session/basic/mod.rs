use crate::session::{Session, SessionManagerType};
use crate::system::auth_token::Model as AuthToken;
use async_trait::async_trait;
use rand::distributions::Alphanumeric;
use rand::Rng;
use std::collections::HashMap;
use std::ops::{Add};

use time::{Duration, OffsetDateTime};
use tokio::sync::RwLock;

pub struct BasicSessionManager {
    pub sessions: RwLock<HashMap<String, Session>>,
}

impl Default for BasicSessionManager {
    fn default() -> Self {
        BasicSessionManager {
            sessions: RwLock::new(HashMap::new()),
        }
    }
}

#[async_trait]
impl SessionManagerType for BasicSessionManager {
    type Error = ();

    async fn delete_session(&self, token: &str) -> Result<(), Self::Error> {
        let mut guard = self.sessions.write().await;
        guard.remove(token);
        return Ok(());
    }

    async fn create_session(&self) -> Result<Session, Self::Error> {
        let mut guard = self.sessions.write().await;
        let session = Session {
            token: generate_token(),
            auth_token: None,
            expiration: token_expiration(),
        };
        guard.insert(session.token.clone(), session.clone());
        return Ok(session);
    }

    async fn retrieve_session(&self, token: &str) -> Result<Option<Session>, Self::Error> {
        let guard = self.sessions.read().await;
        for x in guard.iter() {
            println!("{:?}", x.0);
        }
        return Ok(guard.get(token).cloned());
    }

    async fn re_create_session(&self, token: &str) -> Result<Session, Self::Error> {
        let mut guard = self.sessions.write().await;
        guard.remove(token);

        let session = Session {
            token: generate_token(),
            auth_token: None,
            expiration: token_expiration(),
        };
        guard.insert(session.token.clone(), session.clone());
        return Ok(session);
    }

    async fn set_auth_token(&self, token: &str, auth_token: AuthToken) -> Result<(), Self::Error> {
        let mut guard = self.sessions.write().await;

        for x in guard.iter() {
            println!("{:?}", x.0);
        }
        let option = guard.get_mut(token);
        if let Some(x) = option {
            x.auth_token = Some(auth_token);
            return Ok(());
        }

        log::warn!(
            "An AuthToken was set to a session that did not exist! {}",
            token
        );
        return Ok(());
    }
}

fn generate_token() -> String {
    let token: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(12)
        .map(char::from)
        .collect();
    format!("nrs_{}", token)
}

pub fn token_expiration() -> OffsetDateTime {
    OffsetDateTime::now_utc().add(Duration::days(1))
}

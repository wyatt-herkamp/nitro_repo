use std::borrow::Cow;
use std::collections::HashMap;

use async_trait::async_trait;
use chrono::{DateTime, Duration, Local};
use log::trace;
use rand::distributions::Alphanumeric;
use rand::Rng;
use tokio::sync::RwLock;

use crate::authentication::session::{Session, SessionManagerType};
use crate::utils::get_current_time;

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
            user: None,
            expiration: token_expiration(),
        };
        guard.insert(session.token.clone(), session.clone());
        return Ok(session);
    }

    async fn retrieve_session(&self, token: &str) -> Result<Option<Session>, Self::Error> {
        let guard = self.sessions.read().await;
        for (session, value) in guard.iter() {
            trace!("Session {}: {:?}", session, value);
        }
        return Ok(guard.get(token).cloned());
    }

    async fn re_create_session(&self, token: &str) -> Result<Session, Self::Error> {
        let mut guard = self.sessions.write().await;
        guard.remove(token);

        let session = Session {
            token: generate_token(),
            user: None,
            expiration: token_expiration(),
        };
        guard.insert(session.token.clone(), session.clone());
        return Ok(session);
    }

    async fn set_user(&self, token: &str, user: i64) -> Result<(), Self::Error> {
        let mut guard = self.sessions.write().await;

        for x in guard.iter() {
            println!("{:?}", x.0);
        }
        let option = guard.get_mut(token);
        if let Some(x) = option {
            x.user = Some(user);
            return Ok(());
        }

        log::warn!(
            "A user was set to an Auth Token that does not exist! {}",
            token
        );
        return Ok(());
    }

    async fn push_session(&self, session: Session) -> Result<(), Self::Error> {
        let mut guard = self.sessions.write().await;
        guard.insert(session.token.clone(), session);
        return Ok(());
    }

    async fn clean_sessions(&self) -> Result<(), Self::Error> {
        let mut guard = self.sessions.write().await;
        let local: DateTime<Local> = Local::now();
        let mut to_remove = Vec::new();
        for (token, session) in guard.iter() {
            if session.expiration < local {
                to_remove.push(token.clone());
            }
        }
        for token in to_remove {
            guard.remove(&token);
        }
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
#[inline(always)]
pub fn token_expiration() -> DateTime<Local> {
    Local::now() + Duration::days(1)
}

use std::{
    fmt::Debug,
    path::PathBuf,
    sync::atomic::{AtomicBool, Ordering},
};

use crate::{
    app::{
        config::{get_current_directory, Mode},
        NitroRepo,
    },
    error::IntoErrorResponse,
};
use axum::response::{IntoResponse, Response};
use chrono::{DateTime, Duration, FixedOffset, Local};
use http::StatusCode;
use rand::{distr::Alphanumeric, rngs::StdRng, Rng, SeedableRng};
use redb::{CommitError, Database, Error, ReadableTable, ReadableTableMetadata, TableDefinition};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::task::JoinHandle;
use tracing::{
    debug, error,
    field::{display, Empty},
    info, instrument, span, Level,
};
use utoipa::ToSchema;
#[derive(Debug, Error)]
pub enum SessionError {
    #[error("Session not found")]
    RedbError(#[from] redb::Error),
    #[error(transparent)]
    TableError(#[from] redb::TableError),
    #[error(transparent)]
    TransactionError(#[from] redb::TransactionError),
    #[error(transparent)]
    StorageError(#[from] redb::StorageError),
    #[error(transparent)]
    CommitError(#[from] CommitError),
    #[error("Could not parse DateTime: {0}")]
    DateTimeParseError(#[from] chrono::ParseError),
}
impl IntoResponse for SessionError {
    fn into_response(self) -> axum::response::Response {
        error!("{}", self);
        let message = format!(
            "Session Manager Error {:?}. Please Contact the Admin about Session DB Corruption",
            self
        );
        Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(message.into())
            .unwrap()
    }
}
impl IntoErrorResponse for SessionError {
    fn into_response_boxed(self: Box<Self>) -> axum::response::Response {
        (*self).into_response()
    }
}
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SessionManagerConfig {
    #[serde(with = "nr_core::utils::duration_serde::as_seconds")]
    pub lifespan: Duration,
    #[serde(with = "nr_core::utils::duration_serde::as_seconds")]
    pub cleanup_interval: Duration,
    pub database_location: PathBuf,
}
impl Default for SessionManagerConfig {
    fn default() -> Self {
        Self {
            lifespan: Duration::days(1),
            cleanup_interval: Duration::hours(1),
            database_location: get_current_directory().join("sessions.redb"),
        }
    }
}
/// A session type.
/// Stored in the session manager.
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq, ToSchema)]
pub struct Session {
    pub user_id: i32,
    pub session_id: String,
    pub user_agent: String,
    pub ip_address: String,
    pub expires: DateTime<FixedOffset>,
    pub created: DateTime<FixedOffset>,
}
/// A tuple of (user_id, session_id, expires, created)
pub type SessionTuple<'value> = (i32, &'value str, &'value str, &'value str, String, String);
impl Session {
    pub fn new(
        user_id: i32,
        session_id: String,
        user_agent: String,
        ip_address: String,
        life: Duration,
    ) -> Self {
        Self {
            user_id,
            session_id,
            user_agent,
            ip_address,
            expires: Local::now().fixed_offset() + life,
            created: Local::now().fixed_offset(),
        }
    }
    pub fn from_tuple(tuple: SessionTuple) -> Result<Self, SessionError> {
        let (user_id, session_id, user_agent, ip_addr, expires, created) = tuple;

        let expires = DateTime::<FixedOffset>::parse_from_rfc3339(&expires).inspect_err(|err| {
            error!(
                "Failed to parse expires. Delete the Sessions Database: {:?}",
                err
            );
        })?;
        let created = DateTime::<FixedOffset>::parse_from_rfc3339(&created).inspect_err(|err| {
            error!(
                "Failed to parse created. Delete the Sessions Database: {:?}",
                err
            );
        })?;
        let session = Session {
            user_id,
            session_id: session_id.to_owned(),
            user_agent: user_agent.to_owned(),
            ip_address: ip_addr.to_owned(),
            expires,
            created,
        };
        Ok(session)
    }
    pub fn as_tuple_ref(&self) -> SessionTuple {
        (
            self.user_id,
            self.session_id.as_str(),
            self.user_agent.as_str(),
            self.ip_address.as_str(),
            self.expires.to_rfc3339(),
            self.created.to_rfc3339(),
        )
    }
}
const TABLE: TableDefinition<&str, SessionTuple> = TableDefinition::new("sessions");

pub struct SessionManager {
    config: SessionManagerConfig,
    sessions: Database,
    mode: Mode,
    running: AtomicBool,
}
impl Debug for SessionManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SessionManager")
            .field("config", &self.config)
            .field("number_of_sessions", &self.number_of_sessions().ok())
            .field("mode", &self.mode)
            .field("running", &self.running.load(Ordering::Relaxed))
            .finish()
    }
}
impl SessionManager {
    pub fn new(session_config: SessionManagerConfig, mode: Mode) -> Result<Self, Error> {
        let sessions = if session_config.database_location.exists() {
            let database = Database::open(&session_config.database_location)?;
            if mode == Mode::Debug {
                println!("Opened database: {:?}", database);
                let session = database.begin_write()?;
                let table = session.open_table(TABLE)?;
                debug!("Found {} sessions", table.len()?);
            }
            database
        } else {
            Database::create(&session_config.database_location)?
        };

        Ok(Self {
            config: session_config,
            sessions,
            mode,
            running: AtomicBool::new(false),
        })
    }
    pub fn number_of_sessions(&self) -> Result<u64, SessionError> {
        let sessions = self.sessions.begin_read()?;
        let table = sessions.open_table(TABLE)?;
        let len = table.len()?;
        Ok(len)
    }
    pub fn filter_table<F>(
        &self,
        continue_on_err: bool,
        filter: F,
    ) -> Result<Vec<Session>, SessionError>
    where
        F: Fn(&Session) -> bool,
    {
        let sessions = self.sessions.begin_read()?;
        let table = sessions.open_table(TABLE)?;
        let mut sessions = Vec::new();
        for index in table.iter()? {
            let value = match index {
                Ok((_, value)) => value,
                Err(err) => {
                    error!("Failed to iterate over sessions: {:?}", err);
                    if !continue_on_err {
                        return Err(err.into());
                    }
                    continue;
                }
            };
            let session = match Session::from_tuple(value.value()) {
                Ok(ok) => ok,
                Err(err) => {
                    error!("Failed to parse session: {:?}", err);
                    if !continue_on_err {
                        return Err(err);
                    }
                    continue;
                }
            };
            if filter(&session) {
                sessions.push(session);
            }
        }
        Ok(sessions)
    }
    #[instrument]
    pub fn clean_inner(&self) -> Result<u32, SessionError> {
        let mut sessions_removed = 0u32;
        let now = Local::now();
        let to_remove = self.filter_table(true, |session| session.expires < now)?;
        if self.mode.is_debug() {
            debug!(?to_remove, "Sessions to remove");
        }
        let sessions = self.sessions.begin_write()?;
        {
            let mut table = sessions.open_table(TABLE)?;
            for key in to_remove {
                debug!("Removing session: {:?}", key);
                match table.remove(&*key.session_id) {
                    Ok(ok) => {
                        if self.mode == Mode::Debug {
                            let ok = ok.map(|x| Session::from_tuple(x.value()));
                            debug!("Removed session: {:?}", ok);
                        }
                        sessions_removed += 1;
                    }
                    Err(err) => {
                        error!("Failed to remove session: {:?}", err);
                    }
                }
            }
        }
        sessions.commit()?;
        Ok(sessions_removed)
    }
    pub async fn cleaner_task(this: NitroRepo, how_often: std::time::Duration) {
        let session_manager = this.session_manager.clone();

        while session_manager.running.load(Ordering::Relaxed) {
            let sleep_for = {
                let span = span!(
                    Level::INFO,
                    "Session Cleaner",
                    sessions.removed = Empty,
                    session.cleaner.error = Empty
                );
                let _enter = span.enter();

                info!("Cleaning sessions");
                match session_manager.clean_inner() {
                    Ok(value) => {
                        info!("Cleaned {} sessions", value);
                        span.record("sessions.removed", value);
                        how_often
                    }
                    Err(err) => {
                        error!("Failed to clean sessions: {:?}", err);
                        span.record("session.cleaner.error", display(err));
                        how_often / 2
                    }
                }
            };
            if let Ok(number_of_sessions) = session_manager.number_of_sessions() {
                this.metrics
                    .active_sessions
                    .add(number_of_sessions as i64, &[]);
            }
            tokio::time::sleep(sleep_for).await
        }
    }
    pub fn start_cleaner(this: NitroRepo) -> Option<JoinHandle<()>> {
        let how_often = this
            .session_manager
            .config
            .cleanup_interval
            .to_std()
            .expect("Duration is too large");
        debug!("Starting Session Cleaner with interval: {:?}", how_often);
        this.session_manager.running.store(true, Ordering::Relaxed);
        let result = tokio::spawn(async move {
            let this = this;
            SessionManager::cleaner_task(this, how_often).await;
        });
        Some(result)
    }
    #[instrument]
    pub fn create_session(
        &self,
        user_id: i32,
        user_agent: String,
        ip_address: String,
        life: Duration,
    ) -> Result<Session, SessionError> {
        let sessions = self.sessions.begin_write()?;
        let mut session_table = sessions.open_table(TABLE)?;

        let session_id =
            create_session_id(|x| session_table.get(x).map(|x| x.is_some()).unwrap_or(false));
        let session = Session::new(user_id, session_id.clone(), user_agent, ip_address, life);

        session_table.insert(&*session_id, session.as_tuple_ref())?;
        drop(session_table);
        sessions.commit()?;
        Ok(session)
    }
    #[instrument]
    pub fn create_session_default_lifespan(
        &self,
        user_id: i32,
        user_agent: String,
        ip_address: String,
    ) -> Result<Session, SessionError> {
        self.create_session(user_id, user_agent, ip_address, self.config.lifespan)
    }
    #[instrument]
    pub fn get_session(&self, session_id: &str) -> Result<Option<Session>, SessionError> {
        let sessions = self.sessions.begin_read()?;

        let session = sessions.open_table(TABLE)?;
        let session = session
            .get(session_id)?
            .map(|x| Session::from_tuple(x.value()))
            .transpose()?;
        Ok(session)
    }
    #[instrument]
    pub fn delete_session(&self, session_id: &str) -> Result<Option<Session>, SessionError> {
        let sessions = self.sessions.begin_write()?;
        let mut table = sessions.open_table(TABLE)?;
        let session = table
            .remove(session_id)?
            .map(|x| Session::from_tuple(x.value()))
            .transpose()?;
        drop(table);
        sessions.commit()?;
        Ok(session)
    }
    pub fn shutdown(&self) {
        self.running.store(false, Ordering::Relaxed);
    }
}

#[inline(always)]
pub fn create_session_id(exists_call_back: impl Fn(&str) -> bool) -> String {
    let mut rand = StdRng::from_os_rng();
    loop {
        let session_id: String = (0..7).map(|_| rand.sample(Alphanumeric) as char).collect();
        if !exists_call_back(&session_id) {
            break session_id;
        }
    }
}

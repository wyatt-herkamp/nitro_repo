//! npm's browser login flow.
//!
//! npm 9 and later default to `--auth-type=web`, so this is what a plain `npm login` uses. It
//! returned `418 I'm a teapot` — the `WebLoginResponse` below was declared and never constructed —
//! which meant login out of the box did not work at all, and a user had to know to pass
//! `--auth-type=legacy` to reach the CouchDB path next door.
//!
//! The exchange:
//!
//! 1. npm `POST`s `/-/v1/login`. The registry answers `200` with a `loginUrl` to open in a browser
//!    and a `doneUrl` to poll.
//! 2. The user opens `loginUrl`, authenticates against the site normally, and approves. The
//!    frontend calls `POST /api/npm/login/{session}`, which mints the repository token.
//! 3. npm polls `doneUrl`. It gets `202` with a `retry-after` while the session is pending, and
//!    `200 {"token": ...}` once approved. The token is handed out exactly once.
//!
//! Sessions live in memory. A restart drops any in flight, which costs the user one re-run of
//! `npm login` — the alternative is persisting short-lived pre-authentication state, which is a
//! worse trade.
use std::time::{Duration, Instant};

use ahash::HashMap;
use axum::response::{IntoResponse, Response};
use http::StatusCode;
use nr_storage::Storage;
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use tracing::{debug, info, instrument};
use uuid::Uuid;

use crate::repository::{
    RepoResponse, RepositoryRequest,
    npm::{NPMRegistryError, utils::NpmRegistryExt},
};

/// How long a user has to finish logging in before the session is discarded.
const SESSION_TTL: Duration = Duration::from_secs(15 * 60);
/// What the registry asks npm to wait between polls of `doneUrl`.
const POLL_INTERVAL_SECONDS: u64 = 5;

#[derive(Debug, Serialize, Deserialize)]
pub struct WebLoginResponse {
    #[serde(rename = "doneUrl")]
    pub done_url: String,
    #[serde(rename = "loginUrl")]
    pub login_url: String,
}

/// What npm sends when it opens a login session. Every field is best-effort, so none is required.
#[derive(Debug, Default, Deserialize)]
pub struct WebLoginRequest {
    #[serde(default)]
    pub hostname: Option<String>,
}

#[derive(Debug)]
struct WebLoginSession {
    repository: Uuid,
    /// Set once a user has approved the session. Taken by the first successful poll.
    token: Option<String>,
    created: Instant,
}

/// The in-memory set of login sessions awaiting a browser.
#[derive(Debug, Default)]
pub struct NpmWebLoginManager {
    sessions: Mutex<HashMap<Uuid, WebLoginSession>>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum PollResult {
    /// No such session, or it expired.
    Unknown,
    /// Waiting on the user.
    Pending,
    /// Approved. The token is yielded once and the session is dropped.
    Ready(String),
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum CompleteError {
    #[error("No such login session, or it has expired")]
    UnknownSession,
    #[error("This login session belongs to a different repository")]
    WrongRepository,
}

impl NpmWebLoginManager {
    pub fn start(&self, repository: Uuid) -> Uuid {
        let id = Uuid::new_v4();
        let mut sessions = self.sessions.lock();
        // Cheapest place to expire: every new session already takes the lock, and the map only
        // grows when someone runs `npm login`.
        sessions.retain(|_, session| session.created.elapsed() < SESSION_TTL);
        sessions.insert(
            id,
            WebLoginSession {
                repository,
                token: None,
                created: Instant::now(),
            },
        );
        id
    }

    /// Which repository a session was opened against, so the approving user can be shown it.
    pub fn repository_for(&self, id: Uuid) -> Option<Uuid> {
        let sessions = self.sessions.lock();
        sessions
            .get(&id)
            .filter(|session| session.created.elapsed() < SESSION_TTL)
            .map(|session| session.repository)
    }

    /// Attaches a minted token to a session.
    ///
    /// The repository is re-checked here rather than trusted from the caller: the session id is
    /// the only thing the browser carries, and it must not be usable to plant a token against a
    /// repository the session was not opened for.
    pub fn complete(&self, id: Uuid, repository: Uuid, token: String) -> Result<(), CompleteError> {
        let mut sessions = self.sessions.lock();
        let Some(session) = sessions
            .get_mut(&id)
            .filter(|session| session.created.elapsed() < SESSION_TTL)
        else {
            return Err(CompleteError::UnknownSession);
        };
        if session.repository != repository {
            return Err(CompleteError::WrongRepository);
        }
        session.token = Some(token);
        Ok(())
    }

    pub fn poll(&self, id: Uuid, repository: Uuid) -> PollResult {
        let mut sessions = self.sessions.lock();
        let Some(session) = sessions.get(&id) else {
            return PollResult::Unknown;
        };
        if session.created.elapsed() >= SESSION_TTL || session.repository != repository {
            sessions.remove(&id);
            return PollResult::Unknown;
        }
        if session.token.is_some() {
            // Removing on read means a token is handed out once. A second poll gets `Unknown`,
            // which npm surfaces as a failed login rather than silently reusing a token.
            let session = sessions.remove(&id).expect("checked above");
            return PollResult::Ready(session.token.expect("checked above"));
        }
        PollResult::Pending
    }
}

/// Opens a login session. `POST /-/v1/login`.
#[instrument(skip(repository, request))]
pub async fn perform_login(
    repository: &impl NpmRegistryExt,
    request: RepositoryRequest,
) -> Result<RepoResponse, NPMRegistryError> {
    let site = repository.site();
    // npm sends a JSON body naming the host it is logging in to. It is advisory, and older
    // clients send nothing at all, so a body that will not parse is not a reason to refuse.
    let body = request.body.body_as_string().await.unwrap_or_default();
    let details: WebLoginRequest = serde_json::from_str(&body).unwrap_or_default();
    debug!(?details, "Starting npm web login");

    let session = site.npm_web_logins.start(repository.id());

    let base = {
        let instance = site.instance.lock();
        instance.app_url.trim_end_matches('/').to_owned()
    };
    if base.is_empty() {
        // Both URLs go to a browser and to npm, so a relative one is useless. Saying so beats
        // handing out a link to nowhere.
        info!("`app_url` is not configured, so npm web login cannot build absolute URLs");
        return Ok(RepoResponse::basic_text_response(
            StatusCode::SERVICE_UNAVAILABLE,
            "This instance has no `app_url` configured, so browser login is unavailable. \
             Set one, or run `npm login --auth-type=legacy`.",
        ));
    }
    let storage = repository.get_storage();
    let storage_name = storage
        .storage_config()
        .storage_config
        .storage_name
        .to_string();
    let repository_base = format!(
        "{}/repositories/{}/{}",
        base,
        storage_name,
        repository.name()
    );

    let response = WebLoginResponse {
        login_url: format!("{base}/npm/login/{session}"),
        done_url: format!("{repository_base}/-/v1/done/{session}"),
    };
    debug!(?response, "Opened npm web login session");
    Ok(RepoResponse::Other(
        Response::builder()
            .status(StatusCode::OK)
            .header(http::header::CONTENT_TYPE, "application/json")
            .body(serde_json::to_string(&response).unwrap().into())
            .unwrap(),
    ))
}

/// The `doneUrl` npm polls. `GET /-/v1/done/{session}`.
#[instrument(skip(repository))]
pub async fn poll_login(
    repository: &impl NpmRegistryExt,
    session: &str,
) -> Result<RepoResponse, NPMRegistryError> {
    let Ok(session) = Uuid::parse_str(session) else {
        return Ok(RepoResponse::basic_text_response(
            StatusCode::NOT_FOUND,
            "Unknown login session",
        ));
    };
    match repository
        .site()
        .npm_web_logins
        .poll(session, repository.id())
    {
        PollResult::Ready(token) => {
            info!("npm web login completed");
            Ok(RepoResponse::Other(
                Response::builder()
                    .status(StatusCode::OK)
                    .header(http::header::CONTENT_TYPE, "application/json")
                    .body(serde_json::json!({ "token": token }).to_string().into())
                    .unwrap(),
            ))
        }
        // npm treats 202 as "keep waiting" and honours `retry-after`.
        PollResult::Pending => Ok(RepoResponse::Other(
            Response::builder()
                .status(StatusCode::ACCEPTED)
                .header(http::header::RETRY_AFTER, POLL_INTERVAL_SECONDS)
                .body(axum::body::Body::empty())
                .unwrap(),
        )),
        PollResult::Unknown => Ok(RepoResponse::basic_text_response(
            StatusCode::NOT_FOUND,
            "Unknown or expired login session",
        )),
    }
}

impl IntoResponse for CompleteError {
    fn into_response(self) -> Response {
        Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(self.to_string().into())
            .unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_token_is_handed_out_once() {
        let manager = NpmWebLoginManager::default();
        let repository = Uuid::new_v4();
        let session = manager.start(repository);

        assert_eq!(manager.poll(session, repository), PollResult::Pending);
        manager
            .complete(session, repository, "token".to_owned())
            .unwrap();
        assert_eq!(
            manager.poll(session, repository),
            PollResult::Ready("token".to_owned())
        );
        // A replayed poll must not hand the same token out again.
        assert_eq!(manager.poll(session, repository), PollResult::Unknown);
    }

    /// The session id is the only thing the browser carries, so it must not be usable to approve
    /// a login against some other repository.
    #[test]
    fn a_session_is_bound_to_its_repository() {
        let manager = NpmWebLoginManager::default();
        let repository = Uuid::new_v4();
        let other = Uuid::new_v4();
        let session = manager.start(repository);

        assert_eq!(
            manager.complete(session, other, "token".to_owned()),
            Err(CompleteError::WrongRepository)
        );
        assert_eq!(manager.poll(session, other), PollResult::Unknown);
    }

    #[test]
    fn unknown_sessions_are_not_pending() {
        let manager = NpmWebLoginManager::default();
        assert_eq!(
            manager.poll(Uuid::new_v4(), Uuid::new_v4()),
            PollResult::Unknown
        );
        assert_eq!(
            manager.complete(Uuid::new_v4(), Uuid::new_v4(), "t".to_owned()),
            Err(CompleteError::UnknownSession)
        );
    }
}

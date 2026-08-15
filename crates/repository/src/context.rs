//! The slice of the application a repository is allowed to see.
//!
//! A repository used to be handed the whole application state, `NitroRepo`: the trait said
//! `fn site(&self) -> NitroRepo`, so every repository type named the concrete application state and
//! could reach the email service, the session manager and the frontend as easily as the database.
//! That made the four repository types inseparable from the binary.
//!
//! [`SiteContext`] is what they actually use — measured, not guessed: the database, the instance
//! settings, the security settings, the staging directory, the request metrics, and the custom
//! hostname index. The server's `NitroRepo` owns one and derefs to it, so nothing on the
//! application side had to change.
//!
//! # Why the hostname *index* and not a repository lookup
//!
//! The index maps a hostname to a [`Uuid`], and stops there. Resolving that id to a loaded
//! repository is deliberately not reachable from here.
//!
//! A repository holds a `SiteContext`. If the context could resolve repositories it would have to
//! hold the map that owns them — and the repositories in that map hold the context, so the `Arc`
//! cycle would never be collected. In a server that lives until the process exits that looks
//! harmless; in the integration tests, where every `TestServer` builds its own state and its `Drop`
//! runs `DROP DATABASE ... WITH (FORCE)`, it means leaking a live connection pool into a database
//! that is being dropped out from under it.
//!
//! Every question a repository actually asks of the index is answerable from the id alone — see
//! [`SiteContext::hostname_belongs_to`], which is the whole of it.

use std::{ops::Deref, sync::Arc};

use ahash::HashMap;
use http::request::Parts;
use nr_web_core::{
    config::{Instance, SecuritySettings},
    utils::host,
};
use parking_lot::{Mutex, RwLock};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{StagingConfig, repo_tracing::RepositoryMetricsMeter};

/// Custom hostnames that route straight into a repository, keyed by the normalised
/// (lowercased, port-stripped) host.
///
/// Materialised in full at startup and kept in step by every mutation, rather than filled lazily
/// like the name lookup table. This one is consulted by the router fallback, which every request
/// that is not `/api`, `/badge` or `/repositories` reaches — including every static asset of the
/// web UI. A lazy read-through cache would turn each of those into an unauthenticated database
/// round trip. Here a miss is authoritative.
#[derive(Debug, Default)]
pub struct HostnameIndex(RwLock<HashMap<String, Uuid>>);

impl HostnameIndex {
    /// The repository registered to `host`, which must already be normalised by
    /// [`nr_web_core::utils::host::normalize_host`].
    pub fn get(&self, host: &str) -> Option<Uuid> {
        self.0.read().get(host).copied()
    }

    pub fn insert(&self, hostname: String, repository: Uuid) {
        self.0.write().insert(hostname.to_lowercase(), repository);
    }

    pub fn remove(&self, hostname: &str) {
        self.0.write().remove(&hostname.to_lowercase());
    }

    /// Drops every hostname pointing at this repository.
    ///
    /// The database rows go by `ON DELETE CASCADE`; this is the in-memory half of the same delete.
    pub fn forget_repository(&self, repository: Uuid) {
        self.0.write().retain(|_, value| *value != repository);
    }

    /// Replaces the whole index, which is what startup does once the database has been read.
    pub fn replace_all(&self, pairs: impl IntoIterator<Item = (String, Uuid)>) {
        let mut table = self.0.write();
        table.clear();
        for (hostname, repository) in pairs {
            table.insert(hostname.to_lowercase(), repository);
        }
    }

    pub fn len(&self) -> usize {
        self.0.read().len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

#[derive(Debug)]
pub struct SiteContextInner {
    pub database: PgPool,
    /// Mutable: installation flips `is_installed`, and the app URL can be rewritten at runtime.
    /// Read through the lock every time rather than snapshotted — see [`SiteContext::app_url`].
    pub instance: Mutex<Instance>,
    pub general_security_settings: SecuritySettings,
    pub staging_config: StagingConfig,
    pub repository_metrics: RepositoryMetricsMeter,
    pub hostnames: HostnameIndex,
}

/// A cheap handle to [`SiteContextInner`]. Cloning one is an `Arc` bump.
#[derive(Debug, Clone)]
pub struct SiteContext(Arc<SiteContextInner>);

impl Deref for SiteContext {
    type Target = SiteContextInner;

    fn deref(&self) -> &SiteContextInner {
        &self.0
    }
}

/// What the ~90 `site.as_ref()` calls in repository code resolve to. `NitroRepo` derived the same
/// impl from its `database` field, so those call sites read the same after the swap.
impl AsRef<PgPool> for SiteContext {
    fn as_ref(&self) -> &PgPool {
        &self.0.database
    }
}

/// Lets the authentication extractors ask for the pool and nothing else, which is all any of them
/// reads. Legal despite `FromRef` and `PgPool` both being foreign: `SiteContext` is local and
/// appears as a trait argument.
impl axum::extract::FromRef<SiteContext> for PgPool {
    fn from_ref(context: &SiteContext) -> PgPool {
        context.0.database.clone()
    }
}

impl SiteContext {
    pub fn new(
        database: PgPool,
        instance: Instance,
        general_security_settings: SecuritySettings,
        staging_config: StagingConfig,
        repository_metrics: RepositoryMetricsMeter,
    ) -> Self {
        Self(Arc::new(SiteContextInner {
            database,
            instance: Mutex::new(instance),
            general_security_settings,
            staging_config,
            repository_metrics,
            hostnames: HostnameIndex::default(),
        }))
    }

    pub fn db(&self) -> &PgPool {
        &self.0.database
    }

    /// The configured app URL, without its trailing slash. Empty when none is configured.
    ///
    /// Takes the lock on every call on purpose. Snapshotting it into the context at construction
    /// would be cheaper and would silently make runtime updates to the app URL invisible to
    /// everything that builds a URL — the cargo index, the Docker token realm, the npm login page.
    pub fn app_url(&self) -> String {
        let instance = self.0.instance.lock();
        instance.app_url.trim_end_matches('/').to_owned()
    }

    pub fn trust_forwarded_host(&self) -> bool {
        self.0.general_security_settings.trust_forwarded_host
    }

    /// The host this request was addressed to, normalised for lookup.
    pub fn request_host(&self, parts: &Parts) -> Option<String> {
        host::request_host(&parts.headers, &parts.uri, self.trust_forwarded_host())
    }

    /// `{scheme}://{host}` for the request as the client addressed it, falling back to the app URL.
    pub fn request_origin(&self, parts: &Parts) -> Option<String> {
        host::request_origin(
            &parts.headers,
            &parts.uri,
            self.trust_forwarded_host(),
            &self.app_url(),
        )
    }

    /// Whether `host` is a custom domain registered to `repository`.
    ///
    /// The whole of what a repository needs to know about hostname routing, and answerable without
    /// resolving anything — which is what keeps the repository map out of the context. See the
    /// module docs.
    pub fn hostname_belongs_to(&self, host: &str, repository: Uuid) -> bool {
        self.0.hostnames.get(host) == Some(repository)
    }
}

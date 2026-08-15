use std::{fmt::Debug, path::PathBuf, sync::Arc};

use ahash::{HashMap, HashMapExt};
use anyhow::Context;
use authentication::session::{SessionManager, SessionManagerConfig};
use axum::extract::State;
use config::{Mode, PasswordRules, SecuritySettings, SiteSetting};
use derive_more::{AsRef, derive::Deref};
use email::EmailSetting;
use email_service::{EmailAccess, EmailService};
use http::{HeaderName, Uri};
pub mod frontend;
pub mod resources;
use nr_core::{
    database::{
        DatabaseConfig,
        entities::{
            repository::{DBRepository, DBRepositoryHostname},
            storage::{DBStorage, StorageDBType},
            user::user_utils,
        },
    },
    repository::config::{
        RepositoryConfigType, project::ProjectConfigType, repository_page::RepositoryPageType,
    },
};
use nr_storage::{DynStorage, STORAGE_FACTORIES, Storage, StorageConfig, StorageFactory};
use opentelemetry::{
    InstrumentationScope, global,
    metrics::{Histogram, Meter, UpDownCounter},
};
use parking_lot::{Mutex, RwLock};
use serde::{Deserialize, Serialize};
pub mod authentication;
pub mod config;
pub mod email;
pub mod email_service;
pub mod request_logging;
use current_semver::current_semver;
use sqlx::PgPool;
use tokio::task::JoinHandle;
use tracing::{debug, info, instrument, warn};
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;
pub mod open_api;
use crate::{
    repository::{
        DynRepository, RepositoryType, RepositoryTypeRegistry, StagingConfig,
        cargo::{CargoRegistryConfigType, CargoRegistryType},
        docker::{DockerRegistryConfigType, DockerRegistryType},
        maven::{MavenPushRulesConfigType, MavenRepositoryConfigType, MavenRepositoryType},
        npm::{NPMRegistryConfigType, NpmRegistryType},
        repo_tracing::RepositoryMetricsMeter,
    },
    utils::ip_addr::HasForwardedHeader,
};
pub mod api;
pub mod badge;
pub mod host_routing;
pub mod responses;
mod site_context;
pub use site_context::{HostnameIndex, SiteContext, SiteContextInner};
pub mod web;
#[derive(Debug, Serialize, Clone, ToSchema)]
pub struct Instance {
    pub app_url: String,
    pub name: String,
    pub description: String,
    pub is_https: bool,
    pub is_installed: bool,
    #[schema(value_type=String)]
    pub version: semver::Version,
    pub mode: Mode,
    pub password_rules: Option<PasswordRules>,
}
#[derive(Debug, Clone, Hash, PartialEq, Eq, IntoParams, Deserialize)]
#[into_params(parameter_in = Path)]
pub struct RepositoryStorageName {
    /// The name of the storage
    pub storage_name: String,
    /// The name of the repository
    pub repository_name: String,
}

impl RepositoryStorageName {
    pub async fn query_db(&self, database: &PgPool) -> Result<Option<Uuid>, sqlx::Error> {
        let query: Option<Uuid> = sqlx::query_scalar(
            r#"SELECT repositories.id FROM repositories INNER JOIN storages
                    ON storages.id = repositories.storage_id AND storages.name = $1
                    WHERE repositories.name = $2"#,
        )
        .bind(&self.storage_name)
        .bind(&self.repository_name)
        .fetch_optional(database)
        .await?;
        Ok(query)
    }
}
impl From<(&str, &str)> for RepositoryStorageName {
    fn from((storage_name, repository_name): (&str, &str)) -> Self {
        Self {
            storage_name: storage_name.to_lowercase(),
            repository_name: repository_name.to_lowercase(),
        }
    }
}
impl From<(String, String)> for RepositoryStorageName {
    fn from((storage_name, repository_name): (String, String)) -> Self {
        Self {
            storage_name: storage_name.to_lowercase(),
            repository_name: repository_name.to_lowercase(),
        }
    }
}
#[derive(Debug, Default)]
pub struct InternalServices {
    pub session_cleaner: Option<JoinHandle<()>>,
    pub email: Option<EmailService>,
}
pub struct NitroRepoInner {
    /// The slice of this state a repository is given. See [`SiteContext`] for what is in it and,
    /// more to the point, what is deliberately not.
    ///
    /// `NitroRepoInner` derefs to its contents, so `site.instance`, `site.staging_config` and
    /// friends still resolve as before.
    pub context: SiteContext,
    /// Every repository type this instance knows about, assembled by
    /// [`default_repository_types`] at startup.
    pub repository_types: RepositoryTypeRegistry,
    pub storages: RwLock<HashMap<Uuid, DynStorage>>,
    pub repositories: RwLock<HashMap<Uuid, DynRepository>>,
    pub name_lookup_table: Mutex<HashMap<RepositoryStorageName, Uuid>>,
    #[cfg(feature = "frontend")]
    pub frontend: frontend::HostedFrontend,
    services: Mutex<InternalServices>,
    pub suggested_local_storage_path: PathBuf,
}

// `std::ops::Deref` spelled out: `Deref` in this module is `derive_more`'s derive macro.
impl std::ops::Deref for NitroRepoInner {
    type Target = SiteContextInner;

    fn deref(&self) -> &SiteContextInner {
        &self.context
    }
}
macro_rules! take_service {
    ($(
        $fn_name:ident => $field:ident -> $type:ty
    ),*) => {
        $(
            pub fn $fn_name(&self) -> Option<$type> {
                let mut services = self.services.lock();
                services.$field.take()
            }
        )*
    }
}
impl NitroRepoInner {
    take_service! {
        take_session_cleaner => session_cleaner -> JoinHandle<()>,
        take_email => email -> EmailService
    }
    /// Notifies services that have waiters that the application is shutting down
    pub fn notify_shutdown(&self) {
        let services = self.services.lock();
        if let Some(email) = services.email.as_ref() {
            email.notify_shutdown.notify_waiters();
        }
    }
}
impl Debug for NitroRepo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // TODO: Improve the Debug implementation
        f.debug_struct("NitroRepo")
            .field("instance", &self.inner.instance.lock())
            .field("active_storages", &self.inner.storages.read().len())
            .field("active_repositories", &self.inner.repositories.read().len())
            .field("database", &self.database)
            .finish()
    }
}
/// Request Metrics based on [HTTP Server Semantic Conventions](https://opentelemetry.io/docs/specs/semconv/http/http-metrics/#http-server)
#[derive(Debug, Clone)]
pub struct AppMetrics {
    pub meter: Meter,
    pub request_size_bytes: Histogram<u64>,
    pub response_size_bytes: Histogram<u64>,
    pub request_duration: Histogram<f64>,
    pub active_sessions: UpDownCounter<i64>,
}
impl Default for AppMetrics {
    fn default() -> Self {
        let scope = InstrumentationScope::builder("nitro-repo")
        .with_schema_url("https://github.com/open-telemetry/semantic-conventions/blob/v1.29.0/docs/http/http-metrics.md")
        .with_version(env!("CARGO_PKG_VERSION")).build();
        let meter = global::meter_with_scope(scope);

        Self {
            active_sessions: meter
                .i64_up_down_counter("http.server.active_sessions")
                .with_description("The number of active sessions")
                .build(),
            request_size_bytes: meter
                .u64_histogram("http.server.request.body.size")
                .with_unit("By")
                .build(),
            response_size_bytes: meter
                .u64_histogram("http.server.response.body.size")
                .with_unit("By")
                .build(),
            request_duration: meter
                .f64_histogram("http.server.request.duration")
                .with_boundaries(vec![
                    0.005, 0.01, 0.025, 0.05, 0.075, 0.1, 0.25, 0.5, 0.75, 1f64, 2.5, 5f64, 7.5,
                    10f64,
                ])
                .with_unit("s")
                .build(),
            meter,
        }
    }
}
#[derive(Clone, AsRef, Deref)]
pub struct NitroRepo {
    #[deref(forward)]
    pub inner: Arc<NitroRepoInner>,
    pub database: PgPool,
    pub session_manager: Arc<SessionManager>,
    pub email_access: Arc<EmailAccess>,
    pub metrics: AppMetrics,
    pub repository_metrics: RepositoryMetricsMeter,
}
static X_FORWARDED_FOR_HEADER: HeaderName = HeaderName::from_static("x-forwarded-for");

impl HasForwardedHeader for NitroRepo {
    fn forwarded_header(&self) -> Option<&http::HeaderName> {
        Some(&X_FORWARDED_FOR_HEADER)
    }
}

/// Lets an extractor ask for only the context, so anything that needs no more than the context is
/// not bound to the whole application state. `RepositoryAuthentication` is the reason this exists.
impl axum::extract::FromRef<NitroRepo> for SiteContext {
    fn from_ref(site: &NitroRepo) -> SiteContext {
        site.inner.context.clone()
    }
}

/// The authentication extractors read nothing but the pool, so they ask for only that.
impl axum::extract::FromRef<NitroRepo> for PgPool {
    fn from_ref(site: &NitroRepo) -> PgPool {
        site.database.clone()
    }
}

impl NitroRepo {
    /// The slice of this state that repositories are given.
    pub fn context(&self) -> SiteContext {
        self.inner.context.clone()
    }
}

impl NitroRepo {
    #[instrument]
    async fn load_database(database: DatabaseConfig) -> anyhow::Result<PgPool> {
        info!(?database, "Connecting to database");
        let options = database.try_into()?;
        info!(?options, "Database connection options");
        let database = PgPool::connect_with(options)
            .await
            .context("Could not connect to database")?;
        nr_core::database::migration::run_migrations(&database).await?;
        Ok(database)
    }
    #[allow(clippy::too_many_arguments)]
    pub async fn new(
        mode: Mode,
        site: SiteSetting,
        security: SecuritySettings,
        session_manager: SessionManagerConfig,
        staging_config: StagingConfig,
        email_settings: Option<EmailSetting>,
        database: DatabaseConfig,
        suggested_local_storage_path: Option<PathBuf>,
    ) -> anyhow::Result<Self> {
        let database = Self::load_database(database).await?;
        let is_installed = user_utils::does_user_exist(&database).await?;
        let instance = Instance {
            mode,
            version: current_semver!(),
            app_url: site.app_url.unwrap_or_default(),
            is_installed,
            name: site.name,
            description: site.description,
            is_https: site.is_https,
            password_rules: security.password_rules.clone(),
        };
        let mut services = InternalServices::default();

        let (email_access, service) = EmailService::start(email_settings).await?;
        services.email = Some(service);
        let suggested_local_storage_path = if let Some(path) = suggested_local_storage_path {
            path
        } else {
            std::env::current_dir()?.join("storages")
        };
        let repository_metrics = RepositoryMetricsMeter::default();
        let context = SiteContext::new(
            database.clone(),
            instance,
            security,
            staging_config,
            repository_metrics.clone(),
        );
        let nitro_repo = NitroRepoInner {
            repository_types: default_repository_types(&context.staging_config.staging_dir),
            context,
            storages: RwLock::new(HashMap::new()),
            repositories: RwLock::new(HashMap::new()),
            name_lookup_table: Mutex::new(HashMap::new()),
            services: Mutex::new(services),
            #[cfg(feature = "frontend")]
            frontend: frontend::HostedFrontend::new(site.frontend_path)?,
            suggested_local_storage_path,
        };

        let session_manager = Arc::new(SessionManager::new(session_manager, mode)?);

        let nitro_repo = NitroRepo {
            inner: Arc::new(nitro_repo),
            session_manager,
            database,
            email_access: Arc::new(email_access),
            metrics: AppMetrics::default(),
            repository_metrics,
        };
        nitro_repo.load_storages().await?;
        nitro_repo.load_repositories().await?;
        nitro_repo.load_hostnames().await?;
        Ok(nitro_repo)
    }

    /// # Notes
    ///
    /// Lock is held intentionally to prevent anything else touching the storages while they are being loaded
    #[allow(clippy::await_holding_lock)]
    async fn load_storages(&self) -> anyhow::Result<()> {
        let mut storages = self.storages.write();
        storages.clear();

        let db_storages = DBStorage::get_all(&self.database).await?;
        let storage_configs = db_storages
            .into_iter()
            .map(StorageConfig::try_from)
            .collect::<Result<Vec<_>, _>>()?;

        for storage_config in storage_configs {
            let id = storage_config.storage_config.storage_id;
            info!(?storage_config, "Loading storage");
            let Some(factory) =
                self.get_storage_factory(&storage_config.storage_config.storage_type)
            else {
                warn!(
                    "No storage factory found for {}",
                    storage_config.storage_config.storage_type
                );
                continue;
            };
            let storage = factory.create_storage(storage_config).await?;
            storages.insert(id, storage);
        }
        info!("Loaded {} storages", storages.len());
        Ok(())
    }
    /// # Notes
    ///
    /// Lock is held intentionally to prevent anything else touching the repositories while they are being loaded
    #[allow(clippy::await_holding_lock)]
    async fn load_repositories(&self) -> anyhow::Result<()> {
        let mut repositories = self.repositories.write();
        repositories.clear();
        let db_repositories = DBRepository::get_all(&self.database).await?;
        for db_repository in db_repositories {
            let storage = self
                .get_storage(db_repository.storage_id)
                .context("Storage not found")?;
            let repository_type = self
                .get_repository_type(&db_repository.repository_type)
                .context("Repository type not found")?;
            let repository_id = db_repository.id;
            let repository = repository_type
                .load_repo(db_repository, storage, self.context())
                .await?;
            repositories.insert(repository_id, repository);
        }
        info!("Loaded {} repositories", repositories.len());
        Ok(())
    }
    /// Fills the [`HostnameIndex`] from the database.
    ///
    /// Must run after [`Self::load_repositories`] — a hostname is only useful once the repository
    /// it points at is loaded.
    async fn load_hostnames(&self) -> anyhow::Result<()> {
        let pairs = DBRepositoryHostname::all_pairs(&self.database).await?;
        self.hostnames.replace_all(pairs);
        info!("Loaded {} repository hostnames", self.hostnames.len());
        Ok(())
    }
    pub fn get_storage_factory(&self, storage_name: &str) -> Option<&'static dyn StorageFactory> {
        STORAGE_FACTORIES
            .iter()
            .find(|factory| factory.storage_name() == storage_name)
            .copied()
    }
    pub async fn close(self) {
        self.session_manager.shutdown();
        self.inner.notify_shutdown();
        //TODO: Close Repositories
        let storages = {
            let mut storages = self.storages.write();
            // Take the values out of the hashmap and clear it
            std::mem::take(&mut *storages)
        };
        for (id, storage) in storages.into_iter() {
            info!(?id, "Unloading storage");
            storage.unload().await.unwrap_or_else(|err| {
                warn!(?id, "Failed to unload storage: {}", err);
            });
        }
        info!("Removing Logger");

        info!("Removing Email");
        let email = self.inner.take_email();
        info!("Email State has been taken");
        if let Some(email) = email {
            email.handle.abort();
        }
        let session_cleaner = self.inner.take_session_cleaner();
        if let Some(handle) = session_cleaner {
            handle.abort();
        }
    }
    pub fn get_repository_config_type(
        &self,
        name: &str,
    ) -> Option<&'static dyn RepositoryConfigType> {
        REPOSITORY_CONFIG_TYPES
            .iter()
            .find(|config_type| config_type.get_type().eq_ignore_ascii_case(name))
            .copied()
    }
    pub fn get_repository(&self, id: Uuid) -> Option<DynRepository> {
        let repository = self.repositories.read();
        repository.get(&id).cloned()
    }
    pub fn add_storage(&self, id: Uuid, storage: DynStorage) {
        let mut storages = self.storages.write();
        storages.insert(id, storage);
    }
    pub fn add_repository(&self, id: Uuid, repository: DynRepository) {
        let mut repositories = self.repositories.write();
        repositories.insert(id, repository);
    }

    pub fn update_app_url(&self, app_url: &Uri) {
        info!(?app_url, "Updating app url");
        // TODO:
    }
    /// Checks if a repository name and storage pair are found in the lookup table. If not queries the database.
    /// If found in the database, adds the pair to the lookup table
    ///
    /// ## Notes
    /// [RepositoryStorageName] is case insensitive. It will be converted to lowercase before being queried. Database queries are case insensitive
    #[instrument(skip(name))]
    pub async fn get_repository_from_names(
        &self,
        name: &RepositoryStorageName,
    ) -> Result<Option<DynRepository>, sqlx::Error> {
        let id = {
            let lookup_table = self.inner.name_lookup_table.lock();
            lookup_table.get(name).cloned()
        };
        if let Some(id) = id {
            debug!(?id, ?name, "Found id in lookup table");
            let repository: Option<DynRepository> = self.get_repository(id);
            if repository.is_none() {
                warn!(?name, "Unregistered database id found in lookup table");
                {
                    let mut lookup_table = self.inner.name_lookup_table.lock();
                    lookup_table.remove(name);
                }
                return Ok(repository);
            }
            return Ok(repository);
        }
        debug!(
            ?name,
            "Name not found in lookup table. Attempting to query database"
        );
        let id = name.query_db(&self.database).await?;
        if let Some(id) = id {
            debug!(?id, ?name, "Found id in database");
            let repository: Option<DynRepository> = self.get_repository(id);
            if repository.is_none() {
                warn!(
                    ?name,
                    "Unregistered database id found. Repositories in database do not match loaded repositories"
                );
                // TODO: Reload Everything
                return Ok(repository);
            }
            // Add the name to the lookup table
            let mut lookup_table = self.inner.name_lookup_table.lock();
            lookup_table.insert(name.clone(), id);

            return Ok(repository);
        }
        // No repository found in the database
        Ok(None)
    }
    /// The repository a request for `host` belongs to, or `None` if the host is not registered.
    ///
    /// `host` must already have been normalised by [`crate::utils::host::normalize_host`].
    ///
    /// Unlike [`Self::get_repository_from_names`] this is infallible and synchronous: the index is
    /// complete, so a miss needs no database round trip. That is what makes it cheap enough to run
    /// on every request that reaches the router's fallback.
    pub fn repository_for_hostname(&self, host: &str) -> Option<DynRepository> {
        let id = self.hostnames.get(host)?;
        let repository = self.get_repository(id);
        if repository.is_none() {
            warn!(?host, ?id, "Hostname points at an unloaded repository");
            self.hostnames.remove(host);
        }
        repository
    }
    pub fn register_hostname(&self, hostname: String, repository_id: Uuid) {
        self.hostnames.insert(hostname, repository_id);
    }
    pub fn unregister_hostname(&self, hostname: &str) {
        self.hostnames.remove(hostname);
    }
    /// Drops every hostname pointing at this repository.
    ///
    /// The database rows go by `ON DELETE CASCADE`; this is the in-memory half of the same delete.
    pub fn forget_repository_hostnames(&self, id: Uuid) {
        self.hostnames.forget_repository(id);
    }
    pub fn get_storage(&self, id: Uuid) -> Option<DynStorage> {
        let storages = self.storages.read();
        storages.get(&id).cloned()
    }
    pub fn get_repository_type(&self, name: &str) -> Option<Arc<dyn RepositoryType>> {
        self.inner.repository_types.get(name)
    }
    /// Drops any cached name that resolves to this repository.
    ///
    /// The lookup table is filled lazily and never invalidated, so after a rename the old name
    /// would keep resolving from cache — the repository would answer on both names until a
    /// restart. Dropping every entry for the id is enough: the next request for the new name
    /// misses, queries the database and re-caches.
    pub fn forget_repository_names(&self, id: Uuid) {
        let mut lookup_table = self.inner.name_lookup_table.lock();
        lookup_table.retain(|_, value| *value != id);
    }
    pub fn remove_repository(&self, id: Uuid) {
        {
            let mut repositories = self.repositories.write();
            repositories.remove(&id);
        }
        {
            let mut lookup_table = self.inner.name_lookup_table.lock();
            lookup_table.retain(|_, value| *value != id);
        }
        self.forget_repository_hostnames(id);
    }
    fn set_session_cleaner(&self, cleaner: JoinHandle<()>) {
        let mut services = self.inner.services.lock();
        services.session_cleaner = Some(cleaner);
    }
    fn start_session_cleaner(&self) {
        // The metric is reported through a callback: `SessionManager` lives in `nr-web-core` and
        // cannot see `AppMetrics`, which belongs to the application.
        let metrics = self.metrics.clone();
        let result = self.session_manager.clone().start_cleaner(move |active| {
            metrics.active_sessions.add(active as i64, &[]);
        });
        if let Some(handle) = result {
            self.set_session_cleaner(handle);
            info!("Session cleaner started");
        }
    }
}

pub type NitroRepoState = State<NitroRepo>;

pub static REPOSITORY_CONFIG_TYPES: &[&dyn RepositoryConfigType] = &[
    &ProjectConfigType,
    &RepositoryPageType,
    &MavenRepositoryConfigType,
    &MavenPushRulesConfigType,
    &NPMRegistryConfigType,
    &CargoRegistryConfigType,
    &DockerRegistryConfigType,
];
/// Every repository type this build knows about.
///
/// Assembled at startup rather than as a `&'static` slice because a type may own state that
/// depends on configuration — Docker's blob-upload manager needs the staging directory.
///
/// `staging_dir` is only read to derive paths; nothing here touches the filesystem, so a caller
/// with no real configuration (the exporter, the tests) can pass a throwaway directory.
pub fn default_repository_types(staging_dir: &std::path::Path) -> RepositoryTypeRegistry {
    RepositoryTypeRegistry::new(vec![
        Arc::new(MavenRepositoryType),
        Arc::new(NpmRegistryType::default()),
        Arc::new(CargoRegistryType),
        Arc::new(DockerRegistryType::new(staging_dir)),
    ])
}

#[cfg(test)]
mod registry_tests {
    use super::*;

    /// The real registry. Nothing here touches the staging directory — `DockerRegistryType::new`
    /// only derives paths from it — so a throwaway one is enough.
    fn test_registry() -> RepositoryTypeRegistry {
        default_repository_types(&std::env::temp_dir())
    }

    /// Every string this server persists or routes on, written out by hand.
    ///
    /// `repositories.repository_type` and `repository_configs.key` are free `VARCHAR`s with no
    /// constraint behind them, and `full_type()` feeds the span and metric attributes that
    /// dashboards are built on. Nothing about renaming a Rust item tells you that you have just
    /// orphaned every repository row of that type — the failure surfaces at startup as
    /// "Repository type not found", naming nothing useful.
    ///
    /// So this test deliberately duplicates the constants rather than referring to them. A
    /// duplicate that has to be edited in two places is the point: the second edit is where you
    /// notice the first one was a data migration.
    #[test]
    fn repository_type_ids_are_stable() {
        use crate::repository::{cargo, docker, maven, npm};

        assert_eq!(maven::REPOSITORY_TYPE_ID, "maven");
        assert_eq!(npm::REPOSITORY_TYPE_ID, "npm");
        assert_eq!(cargo::REPOSITORY_TYPE_ID, "cargo");
        assert_eq!(docker::REPOSITORY_TYPE_ID, "docker");

        assert_eq!(maven::hosted::FULL_TYPE, "maven/hosted");
        assert_eq!(maven::proxy::FULL_TYPE, "maven/proxy");
        assert_eq!(npm::hosted::FULL_TYPE, "npm/hosted");
        assert_eq!(cargo::hosted::FULL_TYPE, "cargo/hosted");
        assert_eq!(docker::hosted::FULL_TYPE, "docker/hosted");

        assert_eq!(MavenRepositoryConfigType.get_type(), "maven");
        assert_eq!(MavenPushRulesConfigType.get_type(), "maven_push_rules");
        assert_eq!(NPMRegistryConfigType.get_type(), "npm");
        assert_eq!(CargoRegistryConfigType.get_type(), "cargo");
        assert_eq!(DockerRegistryConfigType.get_type(), "docker");
        assert_eq!(ProjectConfigType.get_type(), "project");
        // `page`, not `repository_page` — the config key and the Rust type name differ here.
        assert_eq!(RepositoryPageType.get_type(), "page");
    }

    /// The registry is a hand-written list, so a dropped entry is not a compile error — and the
    /// two drift tests below iterate it, so a *missing* type passes both of those happily. The
    /// only symptom would be every repository of that type failing to load at startup.
    #[test]
    fn every_repository_type_is_registered() {
        let mut registered: Vec<_> = test_registry()
            .iter()
            .map(|repository_type| repository_type.get_type())
            .collect();
        registered.sort_unstable();
        assert_eq!(registered, ["cargo", "docker", "maven", "npm"]);
    }

    /// Every config key a repository type advertises must actually be registered.
    ///
    /// `get_repository_config_type` is a linear scan over `REPOSITORY_CONFIG_TYPES`, and a miss is
    /// just `None` — so a type advertising a key nobody registered fails at request time with a
    /// confusing error rather than at startup.
    #[test]
    fn advertised_config_types_are_registered() {
        for repository_type in test_registry().iter() {
            for key in repository_type.config_types() {
                assert!(
                    REPOSITORY_CONFIG_TYPES
                        .iter()
                        .any(|config| config.get_type().eq_ignore_ascii_case(key)),
                    "`{}` advertises config `{key}`, which is not in REPOSITORY_CONFIG_TYPES",
                    repository_type.get_type()
                );
            }
        }
    }

    /// A config that must be supplied at creation has to be one the type actually supports.
    ///
    /// These were separate hand-maintained lists and had already drifted: Maven's `config_types`
    /// omitted `maven`, the very key its `required_configs` demanded.
    #[test]
    fn required_configs_are_a_subset_of_supported_ones() {
        for repository_type in test_registry().iter() {
            let supported = repository_type.config_types();
            for required in repository_type.get_description().required_configs {
                assert!(
                    supported.contains(&required),
                    "`{}` requires config `{required}` but does not list it in config_types: {supported:?}",
                    repository_type.get_type()
                );
            }
        }
    }
}

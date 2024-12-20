use std::{fmt::Debug, sync::Arc};

use ahash::{HashMap, HashMapExt};
use anyhow::Context;
use authentication::session::{SessionManager, SessionManagerConfig};

use axum::extract::State;
use config::{Mode, PasswordRules, SecuritySettings, SiteSetting};
use derive_more::{derive::Deref, AsRef, Into};
use email::EmailSetting;
use email_service::{EmailAccess, EmailService};
use http::Uri;
use nr_core::{
    database::{
        repository::DBRepository,
        storage::{DBStorage, StorageDBType},
        user::user_utils,
        DatabaseConfig,
    },
    repository::config::{
        project::ProjectConfigType, repository_page::RepositoryPageType, RepositoryConfigType,
    },
};
use nr_storage::{DynStorage, Storage, StorageConfig, StorageFactory, STORAGE_FACTORIES};
use parking_lot::{Mutex, RwLock};
use serde::Serialize;
pub mod authentication;
pub mod config;
pub mod email;
pub mod email_service;
pub mod logging;
use current_semver::current_semver;
use sqlx::PgPool;
use tracing::{debug, info, instrument, warn};
use utoipa::ToSchema;
use uuid::Uuid;
pub mod open_api;
use crate::repository::{
    maven::{MavenPushRulesConfigType, MavenRepositoryConfigType, MavenRepositoryType},
    npm::{NPMRegistryConfigType, NpmRegistryType},
    DynRepository, RepositoryType, StagingConfig,
};
pub mod api;
pub mod badge;
pub mod responses;
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
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct RepositoryStorageName {
    pub storage_name: String,
    pub repository_name: String,
}

impl RepositoryStorageName {
    pub async fn query_db(&self, database: &PgPool) -> Result<Option<Uuid>, sqlx::Error> {
        let query: Option<Uuid> = sqlx::query_scalar(
            r#"SELECT repositories.id FROM repositories LEFT JOIN storages
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
pub struct NitroRepoInner {
    pub instance: Mutex<Instance>,
    pub storages: RwLock<HashMap<Uuid, DynStorage>>,
    pub repositories: RwLock<HashMap<Uuid, DynRepository>>,
    pub name_lookup_table: Mutex<HashMap<RepositoryStorageName, Uuid>>,
    pub general_security_settings: SecuritySettings,
    pub staging_config: StagingConfig,
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
#[derive(Clone, AsRef, Deref)]
pub struct NitroRepo {
    #[deref(forward)]
    pub inner: Arc<NitroRepoInner>,
    pub database: PgPool,
    pub session_manager: Arc<SessionManager>,
    pub email_access: Arc<EmailAccess>,
}
impl NitroRepo {
    async fn load_database(database: DatabaseConfig) -> anyhow::Result<PgPool> {
        let database = PgPool::connect_with(database.try_into()?)
            .await
            .context("Could not connec to database")?;
        nr_core::database::migration::run_migrations(&database).await?;
        Ok(database)
    }
    pub async fn new(
        mode: Mode,
        site: SiteSetting,
        security: SecuritySettings,
        session_manager: SessionManagerConfig,
        staging_config: StagingConfig,
        email_settings: Option<EmailSetting>,
        database: DatabaseConfig,
    ) -> anyhow::Result<Self> {
        let database = Self::load_database(database).await?;
        let is_installed = user_utils::does_user_exist(&database).await?;
        let security = security;
        let staging_config = staging_config;
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
        let email_service = if let Some(email_service) = email_settings {
            EmailService::start(email_service).await?
        } else {
            EmailService::no_email()
        };
        let session_manager = SessionManager::new(session_manager, mode)?;

        let nitro_repo = NitroRepoInner {
            instance: Mutex::new(instance),
            storages: RwLock::new(HashMap::new()),
            repositories: RwLock::new(HashMap::new()),
            name_lookup_table: Mutex::new(HashMap::new()),
            general_security_settings: security,
            staging_config,
        };
        let session_manager = Arc::new(session_manager);
        SessionManager::start_cleaner(session_manager.clone());
        let nitro_repo = NitroRepo {
            inner: Arc::new(nitro_repo),
            session_manager,
            database,
            email_access: Arc::new(email_service),
        };
        nitro_repo.load_storages().await?;
        nitro_repo.load_repositories().await?;
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
                .load_repo(db_repository, storage, self.clone())
                .await?;
            repositories.insert(repository_id, repository);
        }
        info!("Loaded {} repositories", repositories.len());
        Ok(())
    }
    pub fn get_storage_factory(&self, storage_name: &str) -> Option<&'static dyn StorageFactory> {
        STORAGE_FACTORIES
            .iter()
            .find(|factory| factory.storage_name() == storage_name)
            .copied()
    }
    #[instrument]
    pub async fn close(&self) {
        self.session_manager.shutdown();
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
        name: impl Into<RepositoryStorageName>,
    ) -> Result<Option<DynRepository>, sqlx::Error> {
        let name = name.into();
        let id = {
            let lookup_table = self.inner.name_lookup_table.lock();
            lookup_table.get(&name).cloned()
        };
        if let Some(id) = id {
            debug!(?id, ?name, "Found id in lookup table");
            let repository: Option<DynRepository> = self.get_repository(id);
            if repository.is_none() {
                warn!(?name, "Unregistered database id found in lookup table");
                {
                    let mut lookup_table = self.inner.name_lookup_table.lock();
                    lookup_table.remove(&name);
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
                warn!(?name, "Unregistered database id found. Repositories in database do not match loaded repositories");
                // TODO: Reload Everything
                return Ok(repository);
            }
            // Add the name to the lookup table
            let mut lookup_table = self.inner.name_lookup_table.lock();
            lookup_table.insert(name, id);

            return Ok(repository);
        }
        // No repository found in the database
        Ok(None)
    }
    pub fn get_storage(&self, id: Uuid) -> Option<DynStorage> {
        let storages = self.storages.read();
        storages.get(&id).cloned()
    }
    pub fn get_repository_type(&self, name: &str) -> Option<&'static dyn RepositoryType> {
        REPOSITORY_TYPES
            .iter()
            .find(|repo_type| repo_type.get_type().eq_ignore_ascii_case(name))
            .copied()
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
    }
}

pub type NitroRepoState = State<NitroRepo>;

pub static REPOSITORY_CONFIG_TYPES: &[&dyn RepositoryConfigType] = &[
    &ProjectConfigType,
    &RepositoryPageType,
    &MavenRepositoryConfigType,
    &MavenPushRulesConfigType,
    &NPMRegistryConfigType,
];
pub static REPOSITORY_TYPES: &[&dyn RepositoryType] = &[&MavenRepositoryType, &NpmRegistryType];

use std::sync::Arc;

use ahash::{HashMap, HashMapExt};
use anyhow::Context;
use authentication::session::{SessionManager, SessionManagerConfig};

use axum::extract::State;
use config::{Mode, PostgresSettings, SecuritySettings, SiteSetting};
use derive_more::{derive::Deref, AsRef, Into};
use http::Uri;
use nr_core::{
    database::{storage::DBStorage, user::does_user_exist},
    repository::config::{
        frontend::{BadgeSettingsType, FrontendConfigType, RepositoryPageType},
        DynRepositoryConfigType, PushRulesConfigType, SecurityConfigType,
    },
};
use nr_storage::{
    local::LocalStorageFactory, DynStorage, DynStorageFactory, Storage, StorageConfig,
    StorageFactory,
};
use parking_lot::{Mutex, RwLock};
use serde::Serialize;

pub mod authentication;
pub mod config;
pub mod email;
pub mod logging;
use current_semver::current_semver;
use sqlx::PgPool;
use tracing::{debug, info, instrument, warn};
use utoipa::ToSchema;
use uuid::Uuid;
pub mod open_api;
use crate::repository::{maven::MavenRepositoryType, DynRepository, DynRepositoryType};
pub mod api;
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
                    ON storages.id = repositories.storage_id
                    WHERE storages.name = ? AND repositories.name = ?"#,
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
#[derive(Debug)]
pub struct NitroRepoInner {
    pub instance: Mutex<Instance>,
    pub storages: RwLock<HashMap<Uuid, DynStorage>>,
    pub repositories: RwLock<HashMap<Uuid, DynRepository>>,
    pub name_lookup_table: Mutex<HashMap<RepositoryStorageName, Uuid>>,
    pub storage_factories: Vec<DynStorageFactory>,
    pub repository_config_types: Vec<DynRepositoryConfigType>,
    pub repository_types: Vec<DynRepositoryType>,
    pub general_security_settings: SecuritySettings,
}
#[derive(Debug, Clone, AsRef, Deref)]
pub struct NitroRepo {
    #[deref(forward)]
    pub inner: Arc<NitroRepoInner>,
    pub database: PgPool,
    pub session_manager: Arc<SessionManager>,
}
impl NitroRepo {
    async fn load_database(database: PostgresSettings) -> anyhow::Result<PgPool> {
        let database = PgPool::connect_with(database.into())
            .await
            .context("Could not connec to database")?;
        sqlx::migrate!()
            .run(&database)
            .await
            .context("Failed to run Migrations")?;
        Ok(database)
    }
    pub async fn new(
        mode: Mode,
        site: SiteSetting,
        security: SecuritySettings,
        session_manager: SessionManagerConfig,
        database: PostgresSettings,
    ) -> anyhow::Result<Self> {
        let database = Self::load_database(database).await?;
        let is_installed = does_user_exist(&database).await?;
        let instance = Instance {
            mode,
            version: current_semver!(),
            app_url: site.app_url.unwrap_or_default(),
            is_installed,
            name: site.name,
            description: site.description,
            is_https: site.is_https,
        };

        let session_manager = SessionManager::new(session_manager, mode)?;

        let factories = vec![LocalStorageFactory::default().into()];
        let nitro_repo = NitroRepoInner {
            instance: Mutex::new(instance),
            storages: RwLock::new(HashMap::new()),
            repositories: RwLock::new(HashMap::new()),
            storage_factories: factories,
            repository_config_types: config_types(),
            repository_types: repository_types(),
            name_lookup_table: Mutex::new(HashMap::new()),
            general_security_settings: security,
        };
        let session_manager = Arc::new(session_manager);
        SessionManager::start_cleaner(session_manager.clone());
        let nitro_repo = NitroRepo {
            inner: Arc::new(nitro_repo),
            session_manager: session_manager,
            database: database,
        };
        nitro_repo.load_storages().await?;
        Ok(nitro_repo)
    }
    ///Unloads all storages and reloads them from the database
    #[instrument]
    async fn load_storages(&self) -> anyhow::Result<()> {
        let mut storages = self.storages.write();
        storages.clear();

        let db_storages = DBStorage::get_all(&self.database).await?;
        let storage_configs = db_storages
            .into_iter()
            .map(|storage| StorageConfig::try_from(storage))
            .collect::<Result<Vec<_>, _>>()?;

        for storage_config in storage_configs {
            let id = storage_config.storage_config.storage_id.clone();
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
    pub fn get_storage_factory(&self, storage_name: &str) -> Option<&DynStorageFactory> {
        self.storage_factories
            .iter()
            .find(|factory| factory.storage_name() == storage_name)
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
    pub fn get_repository_config_type(&self, name: &str) -> Option<&DynRepositoryConfigType> {
        self.repository_config_types
            .iter()
            .find(|config_type| config_type.get_type().eq_ignore_ascii_case(name))
    }
    pub fn get_repository(&self, id: Uuid) -> Option<DynRepository> {
        let repository = self.repositories.read();
        repository.get(&id).cloned()
    }
    pub fn add_storage(&self, id: Uuid, storage: DynStorage) {
        let mut storages = self.storages.write();
        storages.insert(id, storage);
    }

    #[instrument]
    pub fn update_app_url(&self, app_url: &Uri) {
        let mut instance = self.instance.lock();
        if instance.app_url.is_empty() {
            info!("Updating app url to {}", app_url);
            let schema = app_url.scheme_str().unwrap_or("http");
            let host = if let Some(authority) = app_url.host() {
                authority.to_string()
            } else {
                warn!("No host found in uri");
                return;
            };
            instance.app_url = format!("{}://{}", schema, host);
        }
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
}
pub type NitroRepoState = State<NitroRepo>;

fn config_types() -> Vec<DynRepositoryConfigType> {
    vec![
        Box::new(PushRulesConfigType),
        Box::new(SecurityConfigType),
        Box::new(BadgeSettingsType),
        Box::new(FrontendConfigType),
        Box::new(RepositoryPageType),
    ]
}
fn repository_types() -> Vec<DynRepositoryType> {
    vec![Box::new(MavenRepositoryType)]
}

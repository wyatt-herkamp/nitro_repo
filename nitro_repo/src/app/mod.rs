use std::sync::Arc;

use ahash::{HashMap, HashMapExt};
use anyhow::Context;
use authentication::session::{SessionManager, SessionManagerConfig};

use axum::{
    async_trait,
    extract::{FromRef, FromRequestParts, State},
    http::{request::Parts, StatusCode},
    routing::get,
    Router,
};
use config::{PostgresSettings, SecuritySettings, SiteSetting};
use derive_more::{AsRef, Deref, From, Into};
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
use tracing::{info, instrument, warn};
use uuid::Uuid;

use crate::repository::{maven::MavenRepositoryType, DynRepository, DynRepositoryType};

pub mod api;
pub mod web;
#[derive(Debug, Serialize, Clone)]
pub struct Instance {
    pub app_url: String,
    pub name: String,
    pub description: String,
    pub is_https: bool,
    pub is_installed: bool,
    pub version: semver::Version,
}

#[derive(Debug)]
pub struct NitroRepoInner {
    pub instance: Mutex<Instance>,
    pub storages: RwLock<HashMap<Uuid, DynStorage>>,
    pub repositories: RwLock<HashMap<Uuid, DynRepository>>,
    pub storage_factories: Vec<DynStorageFactory>,
    pub repository_config_types: Vec<DynRepositoryConfigType>,
    pub repository_types: Vec<DynRepositoryType>,
    pub session_manager: SessionManager,
    pub general_security_settings: SecuritySettings,
}
#[derive(Debug, Clone)]
pub struct NitroRepo {
    pub inner: Arc<NitroRepoInner>,
    pub database: PgPool,
}
impl AsRef<PgPool> for NitroRepo {
    fn as_ref(&self) -> &PgPool {
        &self.database
    }
}
impl std::ops::Deref for NitroRepo {
    type Target = NitroRepoInner;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
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
        site: SiteSetting,
        security: SecuritySettings,
        session_manager: SessionManagerConfig,
        database: PostgresSettings,
    ) -> anyhow::Result<Self> {
        let database = Self::load_database(database).await?;
        let is_installed = does_user_exist(&database).await?;
        let instance = Instance {
            version: current_semver!(),
            app_url: site.app_url.unwrap_or_default(),
            is_installed,
            name: site.name,
            description: site.description,
            is_https: site.is_https,
        };

        let session_manager = SessionManager::new(session_manager)?;

        let factories = vec![LocalStorageFactory::default().into()];
        let nitro_repo = NitroRepoInner {
            instance: Mutex::new(instance),
            storages: RwLock::new(HashMap::new()),
            repositories: RwLock::new(HashMap::new()),
            storage_factories: factories,
            repository_config_types: config_types(),
            repository_types: repository_types(),
            session_manager: session_manager,
            general_security_settings: security,
        };
        let nitro_repo = NitroRepo {
            inner: Arc::new(nitro_repo),
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

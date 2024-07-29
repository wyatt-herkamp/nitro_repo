use actix_web::web::Data;
use ahash::{HashMap, HashMapExt};
use config::SiteSetting;
use nr_core::{
    database::{storage::DBStorage, user::does_user_exist},
    repository::config::{
        frontend::{BadgeSettingsType, FrontendConfigType, RepositoryPageType},
        DynRepositoryConfigType, PushRulesConfig, PushRulesConfigType, SecurityConfigType,
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

use crate::repository::{
    dyn_repository::DynRepository, maven::MavenRepositoryType, DynRepositoryType,
};

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
pub struct NitroRepo {
    pub instance: Mutex<Instance>,
    pub storages: RwLock<HashMap<Uuid, DynStorage>>,
    pub repositories: RwLock<HashMap<Uuid, DynRepository>>,
    pub storage_factories: Vec<DynStorageFactory>,
    pub repository_config_types: Vec<DynRepositoryConfigType>,
    pub repository_types: Vec<DynRepositoryType>,
}

impl NitroRepo {
    pub async fn new(site: SiteSetting, database: DatabaseConnection) -> anyhow::Result<Self> {
        let is_installed = does_user_exist(&database).await?;
        let instance = Instance {
            version: current_semver!(),
            app_url: site.app_url.unwrap_or_default(),
            is_installed,
            name: site.name,
            description: site.description,
            is_https: site.is_https,
        };
        let factories = vec![LocalStorageFactory::default().into()];
        let nitro_repo = NitroRepo {
            instance: Mutex::new(instance),
            storages: RwLock::new(HashMap::new()),
            repositories: RwLock::new(HashMap::new()),
            storage_factories: factories,
            repository_config_types: config_types(),
            repository_types: repository_types(),
        };
        nitro_repo.load_storages(database).await?;
        Ok(nitro_repo)
    }
    ///Unloads all storages and reloads them from the database
    #[instrument]
    async fn load_storages(&self, database: DatabaseConnection) -> anyhow::Result<()> {
        let mut storages = self.storages.write();
        storages.clear();

        let db_storages = DBStorage::get_all(database.as_ref()).await?;
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
    #[instrument]
    pub fn update_app_url(&self, app_url: String) {
        let mut instance = self.instance.lock();
        if instance.app_url.is_empty() {
            info!("Updating app url to {}", app_url);
            instance.app_url = app_url;
        }
    }
}

pub type DatabaseConnection = Data<PgPool>;
pub type NitroRepoData = Data<NitroRepo>;

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

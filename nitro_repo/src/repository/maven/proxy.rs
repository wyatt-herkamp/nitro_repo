use std::sync::{atomic::AtomicBool, Arc};

use nr_core::{
    database::repository::DBRepository,
    repository::config::{
        frontend::{BadgeSettingsType, FrontendConfigType},
        RepositoryConfigType as _, SecurityConfigType,
    },
};
use nr_storage::DynStorage;
use parking_lot::RwLock;
use uuid::Uuid;

use crate::repository::Repository;

#[derive(Debug)]
pub struct MavenProxyInner {
    pub storage: DynStorage,
    pub id: Uuid,
    pub name: String,
    pub active: AtomicBool,
    pub repository: RwLock<DBRepository>,
}
#[derive(Debug, Clone)]
pub struct MavenProxy(Arc<MavenProxyInner>);

impl Repository for MavenProxy {
    fn get_storage(&self) -> nr_storage::DynStorage {
        self.0.storage.clone()
    }

    fn get_type(&self) -> &'static str {
        "maven"
    }

    fn config_types(&self) -> Vec<&str> {
        vec![
            SecurityConfigType::get_type_static(),
            BadgeSettingsType::get_type_static(),
            FrontendConfigType::get_type_static(),
        ]
    }

    fn name(&self) -> String {
        self.0.name.clone()
    }

    fn id(&self) -> Uuid {
        self.0.id
    }

    fn is_active(&self) -> bool {
        self.0.active.load(std::sync::atomic::Ordering::Relaxed)
    }
}

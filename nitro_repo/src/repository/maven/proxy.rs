use std::sync::Arc;

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

use crate::repository::{Repository, RepositoryFactoryError};

#[derive(Debug)]
pub struct MavenProxyInner {
    pub storage: DynStorage,
    pub id: Uuid,
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

    fn base_config(&self) -> DBRepository {
        self.0.repository.read().clone()
    }
}

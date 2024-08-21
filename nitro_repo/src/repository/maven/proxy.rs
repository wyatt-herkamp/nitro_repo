use std::sync::{atomic::AtomicBool, Arc};

use nr_core::{
    database::repository::DBRepository,
    repository::{
        config::{project::ProjectConfigType, RepositoryConfigType as _, SecurityConfigType},
        Visibility,
    },
};
use nr_storage::DynStorage;
use parking_lot::RwLock;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{app::NitroRepo, repository::Repository};

use super::repo_type::RepositoryFactoryError;
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MavenProxyConfig {
    pub routes: Vec<MavenProxyRepositoryRoute>,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MavenProxyRepositoryRoute {
    pub url: String,
    pub name: Option<String>,
    /// If Null then it will be the lowest priority
    pub priority: Option<i32>,
    // TODO: Credentials
}
#[derive(Debug)]
pub struct MavenProxyInner {
    pub storage: DynStorage,
    pub site: NitroRepo,
    pub id: Uuid,
    pub name: String,
    pub visibility: RwLock<Visibility>,
    pub active: AtomicBool,
    pub config: RwLock<MavenProxyConfig>,
}
#[derive(Debug, Clone)]
pub struct MavenProxy(Arc<MavenProxyInner>);
impl MavenProxy {
    pub async fn load(
        repository: DBRepository,
        storage: DynStorage,
        site: NitroRepo,
        proxy_config: MavenProxyConfig,
    ) -> Result<Self, RepositoryFactoryError> {
        let inner = MavenProxyInner {
            id: repository.id,
            name: repository.name.into(),
            active: AtomicBool::new(repository.active),
            visibility: RwLock::new(repository.visibility),
            config: RwLock::new(proxy_config),
            storage,
            site,
        };
        Ok(Self(Arc::new(inner)))
    }
}

impl Repository for MavenProxy {
    fn get_storage(&self) -> nr_storage::DynStorage {
        self.0.storage.clone()
    }
    fn visibility(&self) -> Visibility {
        Visibility::Public
    }

    fn get_type(&self) -> &'static str {
        "maven"
    }

    fn config_types(&self) -> Vec<&str> {
        vec![
            SecurityConfigType::get_type_static(),
            ProjectConfigType::get_type_static(),
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

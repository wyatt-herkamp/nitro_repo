use std::sync::Arc;

use nr_storage::DynStorage;

use crate::repository::Repository;

#[derive(Debug)]
pub struct MavenProxyInner {
    pub storage: DynStorage,
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

    fn config_types(&self) -> Vec<String> {
        vec![
            "push_rules".to_string(),
            "security".to_string(),
            "maven-proxy".to_string(),
        ]
    }

    fn reload(&self) {}
}

use std::sync::Arc;

use nr_storage::DynStorage;

use crate::repository::Repository;
#[derive(Debug)]
pub struct MavenHostedInner {
    pub storage: DynStorage,
}
#[derive(Debug, Clone)]
pub struct MavenHosted(Arc<MavenHostedInner>);

impl Repository for MavenHosted {
    fn get_storage(&self) -> nr_storage::DynStorage {
        self.0.storage.clone()
    }

    fn get_type(&self) -> &'static str {
        "maven"
    }

    fn config_types(&self) -> Vec<String> {
        vec!["push_rules".to_string(), "security".to_string()]
    }
}

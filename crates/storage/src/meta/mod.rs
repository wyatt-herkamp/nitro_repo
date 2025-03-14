use ahash::HashMap;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RepositoryMeta {
    pub project_id: Option<Uuid>,
    pub project_version_id: Option<Uuid>,
    pub extra_meta: HashMap<String, String>,
}
impl RepositoryMeta {
    pub fn insert(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.extra_meta.insert(key.into(), value.into());
    }
    pub fn has_key(&self, key: &str) -> bool {
        self.extra_meta.contains_key(key)
    }
    pub fn get(&self, key: &str) -> Option<&str> {
        self.extra_meta.get(key).map(|v| v.as_str())
    }

    pub fn set_project_id(&mut self, key: Uuid) {
        self.project_id = Some(key);
    }
    pub fn set_version_id(&mut self, key: Uuid) {
        self.project_version_id = Some(key);
    }
}

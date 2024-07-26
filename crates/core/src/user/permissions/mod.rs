use serde::{Deserialize, Serialize};
use thiserror::Error;

pub mod orm;

#[derive(Error, Debug)]
pub enum PermissionError {
    #[error("Unable to Parse Repository String {0}")]
    ParseError(String),

    #[error("Unable to Parse Storage String")]
    StorageClassifier,
    #[error("Unable to Parse Repository String")]
    RepositoryClassifier,
    #[error("Unable to Parse Repository String {0}")]
    RepositoryClassifierParseError(serde_json::Error),
}

impl From<serde_json::Error> for PermissionError {
    fn from(error: serde_json::Error) -> Self {
        PermissionError::RepositoryClassifierParseError(error)
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, Clone, Default)]
pub struct UserPermissions {
    pub disabled: bool,
    pub admin: bool,
    pub user_manager: bool,
    pub repository_manager: bool,
    #[serde(default)]
    pub deployer: RepositoryPermission,
    #[serde(default)]
    pub viewer: RepositoryPermission,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
pub struct RepositoryPermission {
    pub permissions: Vec<String>,
}
impl Default for RepositoryPermission {
    fn default() -> Self {
        RepositoryPermission {
            permissions: vec!["*".to_string()],
        }
    }
}

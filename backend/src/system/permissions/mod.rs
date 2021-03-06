pub mod options;

use serde::{Serialize, Deserialize};
use crate::repository::models::{Repository};
use thiserror::Error;
use crate::repository::settings::Policy;
use crate::repository::settings::security::Visibility;
use crate::system::permissions::PermissionError::{RepositoryClassifier, StorageClassifier};
use std::io::Write;

use diesel::backend::Backend;
use diesel::deserialize::FromSql;
use diesel::mysql::Mysql;
use diesel::serialize::{Output, ToSql};
use diesel::sql_types::Text;
use diesel::{deserialize, serialize};

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

#[derive(AsExpression, Debug, Deserialize, Serialize, FromSqlRow, Clone, Default)]
#[sql_type = "Text"]
pub struct UserPermissions {
    pub disabled: bool,
    pub admin: bool,
    pub user_manager: bool,
    pub repository_manager: bool,
    pub deployer: Option<RepositoryPermission>,
    pub viewer: Option<RepositoryPermission>,
}

impl UserPermissions {
    pub fn can_access_repository(&self) -> bool {
         self.admin || self.repository_manager
    }
}

impl FromSql<Text, Mysql> for UserPermissions {
    fn from_sql(
        bytes: Option<&<Mysql as Backend>::RawValue>,
    ) -> deserialize::Result<UserPermissions> {
        let t = <String as FromSql<Text, Mysql>>::from_sql(bytes)?;
        let result: UserPermissions = serde_json::from_str(t.as_str())?;
        Ok(result)
    }
}

impl ToSql<Text, Mysql> for UserPermissions {
    fn to_sql<W: Write>(&self, out: &mut Output<W, Mysql>) -> serialize::Result {
        let s = serde_json::to_string(&self)?;
        <String as ToSql<Text, Mysql>>::to_sql(&s, out)
    }
}


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RepositoryPermission {
    pub permissions: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct RepositoryPermissionValue {
    pub policy: Option<Policy>,
    #[serde(rename = "type")]
    pub repo_type: Option<String>,
}



pub fn can_deploy(
    user_perms: &UserPermissions,
    repo: &Repository,
) -> Result<bool, PermissionError> {
    if user_perms.disabled {
        return Ok(false);
    }
    if user_perms.admin {
        return Ok(true);
    }

    if let Some(perms) = &user_perms.deployer {
        return can(repo, perms);
    }
    Ok(false)
}


pub fn can_read(
    user_perms: &UserPermissions,
    repo: &Repository,
) -> Result<bool, PermissionError> {
    if user_perms.disabled {
        return Ok(false);
    }
    if user_perms.admin {
        return Ok(true);
    }

    match repo.security.visibility {
        Visibility::Public => Ok(true),
        Visibility::Private => {
            if let Some(perms) = &user_perms.viewer {
                if can(repo, perms)? {
                    return Ok(true);
                }
            }
            can_deploy(user_perms, repo)
        }
        Visibility::Hidden => Ok(true),
    }
}

pub fn can(repo: &Repository, perms: &RepositoryPermission) -> Result<bool, PermissionError> {
    if perms.permissions.is_empty() {
        // If nothing is set. It is a all view type of scenario
        return Ok(true);
    }
    let repository = repo.name.clone();
    let storage = repo.storage.clone();
    for perm_string in perms.permissions.iter() {
        let split = perm_string.split('/').collect::<Vec<&str>>();
        let storage_perm = split.get(0).ok_or(StorageClassifier)?.to_string();
        if !storage_perm.eq("*") && !storage_perm.eq_ignore_ascii_case(&storage) {
            continue;
        }
        drop(storage_perm);
        let repository_perm = split.get(1).ok_or(RepositoryClassifier)?.to_string();
        if repository_perm.eq("*") || repository_perm.eq(&repository) {
            return Ok(true);
        }
        if repository_perm.starts_with('{') && repository_perm.ends_with('}') {
            let permission: RepositoryPermissionValue = serde_json::from_str(&repository_perm)?;
            if let Some(policy) = &permission.policy {
                if !policy.eq(&repo.settings.policy) {
                    return Ok(false);
                }
            }
            if let Some(repo_type) = &permission.repo_type {
                if !repo_type.eq(&repo.repo_type.to_string()) {
                    return Ok(false);
                }
            }
            return Ok(true);
        }
    }
    Ok(false)
}
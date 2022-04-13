use serde::{Serialize, Deserialize};
use crate::repository::models::{Repository, Visibility};
use thiserror::Error;
use strum_macros::{EnumString, Display};
use crate::system::permissions::PermissionError::{RepositoryClassifier, StorageClassifier};


#[derive(Error, Debug)]
pub enum PermissionError {
    #[error("Unable to Parse Repository String {0}")]
    ParseError(String),

    #[error("Unable to Parse Repository String")]
    StorageClassifier,
    #[error("Unable to Parse Repository String")]
    RepositoryClassifier,
}

pub struct UserPermissions {
    pub disabled: bool,
    pub admin: bool,
    pub user_manager: bool,
    pub repository_manager: bool,
    pub deployer: Option<RepositoryPermission>,
    pub viewer: Option<RepositoryPermission>,
}


#[derive(Serialize, Deserialize, Debug)]
pub struct RepositoryPermission {
    pub permissions: Vec<String>,
}

impl Default for RepositoryPermission {
    fn default() -> Self {
        return RepositoryPermission { permissions: vec![] };
    }
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
            return can_deploy(user_perms, repo);
        }
        Visibility::Hidden => Ok(true),
    }
}

pub fn can(repo: &Repository, perms: &RepositoryPermission) -> Result<bool, PermissionError> {
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
            todo!()
        }
    }
    Ok(false)
}
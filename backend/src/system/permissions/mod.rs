use serde::{Serialize, Deserialize};
use crate::repository::models::{Repository, Visibility};
use thiserror::Error;
use crate::system::permissions::Permission::Admin;
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
    pub permissions: Vec<Permission>,
}

#[derive(Serialize, Deserialize, Debug, EnumString, Display)]
pub enum Permission {
    Admin,
    UserManager,
    RepositoryManager,

    Deployer(RepositoryPermission),
    Viewer(RepositoryPermission),
}

impl PartialEq for Permission {
    fn eq(&self, other: &Self) -> bool {
        if self.to_string().eq(&other.to_string()) {
            return true;
        }
        return false;
    }
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

impl UserPermissions {
    pub fn find_by_name(&self, name: &str) -> Option<&Permission> {
        for perm in self.permissions.iter() {
            if perm.to_string().eq(name) {
                return Some(perm);
            }
        }
        return None;
    }
}

pub fn can_deploy(
    user_perms: &UserPermissions,
    repo: &Repository,
) -> Result<bool, PermissionError> {
    if user_perms.disabled {
        return Ok(false);
    }
    if user_perms.permissions.contains(&Admin) {
        return Ok(true);
    }

    if let Some(view_perms) = user_perms.find_by_name("Deployer") {
        return match view_perms {
            Permission::Viewer(perms) => {
                can(repo, perms)
            }
            _ => {
                //This is literally impossible
                Ok(false)
            }
        };
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
    if user_perms.permissions.contains(&Admin) {
        return Ok(true);
    }
    match repo.security.visibility {
        Visibility::Public => Ok(true),
        Visibility::Private => {
            return if let Some(view_perms) = user_perms.find_by_name("Viewer") {
                match view_perms {
                    Permission::Viewer(perms) => {
                        can(repo, perms)
                    }
                    _ => {
                        //This is literally impossible
                        Ok(false)
                    }
                }
            } else {
                can_deploy(user_perms, repo)
            };
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
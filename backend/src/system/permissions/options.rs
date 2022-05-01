use crate::system::permissions::{can_deploy, can_read, PermissionError, UserPermissions};

use crate::repository::settings::security::Visibility;
use crate::system::user::UserModel;
use std::fmt;

use crate::repository::data::{RepositoryConfig, RepositorySetting};

#[derive(Debug)]
pub struct MissingPermission(String);

impl From<&str> for MissingPermission {
    fn from(value: &str) -> Self {
        MissingPermission(value.to_string())
    }
}

impl fmt::Display for MissingPermission {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Missing permission: {}", self.0)
    }
}

impl std::error::Error for MissingPermission {}

pub trait CanIDo {
    fn can_i_edit_repos(&self) -> Result<(), MissingPermission>;
    fn can_i_edit_users(&self) -> Result<(), MissingPermission>;
    fn can_i_admin(&self) -> Result<(), MissingPermission>;
    fn can_deploy_to<T: RepositorySetting>(
        &self,
        repo: &RepositoryConfig<T>,
    ) -> Result<Option<MissingPermission>, PermissionError>;
    fn can_read_from<T: RepositorySetting>(
        &self,
        repo: &RepositoryConfig<T>,
    ) -> Result<Option<MissingPermission>, PermissionError>;
}

impl CanIDo for UserModel {
    fn can_i_edit_repos(&self) -> Result<(), MissingPermission> {
        let permissions: UserPermissions = self.permissions.clone().try_into().unwrap();

        if !permissions.admin && !permissions.repository_manager {
            return Err("repository_manager".into());
        }
        Ok(())
    }

    fn can_i_edit_users(&self) -> Result<(), MissingPermission> {
        let permissions: UserPermissions = self.permissions.clone().try_into().unwrap();
        if !permissions.admin && !permissions.user_manager {
            return Err("user_manager".into());
        }
        Ok(())
    }

    fn can_i_admin(&self) -> Result<(), MissingPermission> {
        let permissions: UserPermissions = self.permissions.clone().try_into().unwrap();

        if !permissions.admin {
            return Err("admin".into());
        }
        Ok(())
    }

    fn can_deploy_to<T: RepositorySetting>(
        &self,
        repo: &RepositoryConfig<T>,
    ) -> Result<Option<MissingPermission>, PermissionError> {
        let can_read = can_deploy(&self.permissions, repo)?;
        if can_read {
            Ok(None)
        } else {
            Ok(Some(MissingPermission("Write Repository".to_string())))
        }
    }

    fn can_read_from<T: RepositorySetting>(
        &self,
        repo: &RepositoryConfig<T>,
    ) -> Result<Option<MissingPermission>, PermissionError> {
        match repo.main_config.security.visibility {
            Visibility::Public => Ok(None),
            Visibility::Private => {
                let can_read = can_read(&self.permissions, repo)?;
                if can_read {
                    Ok(None)
                } else {
                    Ok(Some(MissingPermission("Read Repository".to_string())))
                }
            }
            Visibility::Hidden => Ok(None),
        }
    }
}

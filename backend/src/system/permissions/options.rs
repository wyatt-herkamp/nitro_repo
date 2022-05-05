use crate::system::permissions::{can_deploy, can_read, PermissionError, UserPermissions};

use crate::api_response::{APIError, APIResponse};
use crate::error::internal_error::InternalError;
use crate::repository::settings::security::Visibility;
use crate::system::user::UserModel;
use actix_web::http::StatusCode;
use std::fmt;

use crate::repository::data::RepositoryConfig;

#[derive(Debug)]
pub struct MissingPermission(String);

impl From<MissingPermission> for APIError {
    fn from(mp: MissingPermission) -> Self {
        APIResponse::from((mp.0, StatusCode::FORBIDDEN)).into()
    }
}
pub trait CanIDo {
    fn can_i_edit_repos(&self) -> Result<(), MissingPermission>;
    fn can_i_edit_users(&self) -> Result<(), MissingPermission>;
    fn can_i_admin(&self) -> Result<(), MissingPermission>;
    fn can_deploy_to(
        &self,
        repo: &RepositoryConfig,
    ) -> Result<Option<MissingPermission>, InternalError>;
    fn can_read_from(
        &self,
        repo: &RepositoryConfig,
    ) -> Result<Option<MissingPermission>, InternalError>;
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

    fn can_deploy_to(
        &self,
        repo: &RepositoryConfig,
    ) -> Result<Option<MissingPermission>, InternalError> {
        let can_read = can_deploy(&self.permissions, repo)?;
        if can_read {
            Ok(None)
        } else {
            Ok(Some(MissingPermission("Write Repository".to_string())))
        }
    }

    fn can_read_from(
        &self,
        repo: &RepositoryConfig,
    ) -> Result<Option<MissingPermission>, InternalError> {
        match repo.visibility {
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

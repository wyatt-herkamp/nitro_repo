use std::fmt;
use crate::system::permissions::UserPermissions;
use crate::system::UserModel;

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
    fn can_i_edit_repos(self) -> Result<UserModel, MissingPermission>;
    fn can_i_edit_users(self) -> Result<UserModel, MissingPermission>;
    fn can_i_admin(self) -> Result<UserModel, MissingPermission>;
}

impl CanIDo for Option<UserModel> {
    fn can_i_edit_repos(self) -> Result<UserModel, MissingPermission> {
        if self.is_none() {
            return Err("Not Logged In".into());
        }
        let user = self.unwrap();
        let permissions: UserPermissions = user.permissions.clone().try_into().unwrap();

        if !permissions.admin && !permissions.repository_manager {
            return Err("repository_manager".into());
        }
        Ok(user)
    }

    fn can_i_edit_users(self) -> Result<UserModel, MissingPermission> {
        if self.is_none() {
            return Err("Not Logged In".into());
        }
        let user = self.unwrap();
        let permissions: UserPermissions = user.permissions.clone().try_into().unwrap();
        if !permissions.admin && !permissions.user_manager {
            return Err("user_manager".into());
        }
        Ok(user)
    }

    fn can_i_admin(self) -> Result<UserModel, MissingPermission> {
        if self.is_none() {
            return Err("Not Logged In".into());
        }
        let user = self.unwrap();
        let permissions: UserPermissions = user.permissions.clone().try_into().unwrap();

        if !permissions.admin {
            return Err("admin".into());
        }
        Ok(user)
    }
}
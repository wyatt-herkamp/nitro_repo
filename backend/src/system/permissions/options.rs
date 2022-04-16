use std::fmt;
use crate::system::models::User;

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
    fn can_i_edit_repos(self) -> Result<User, MissingPermission>;
    fn can_i_edit_users(self) -> Result<User, MissingPermission>;
    fn can_i_admin(self) -> Result<User, MissingPermission>;
}

impl CanIDo for Option<User> {
    fn can_i_edit_repos(self) -> Result<User, MissingPermission> {
        if self.is_none() {
            return Err("Not Logged In".into());
        }
        let user = self.unwrap();
        if !user.permissions.admin && !user.permissions.repository_manager {
            return Err("repository_manager".into());
        }
        Ok(user)
    }

    fn can_i_edit_users(self) -> Result<User, MissingPermission> {
        if self.is_none() {
            return Err("Not Logged In".into());
        }
        let user = self.unwrap();
        if !user.permissions.admin && !user.permissions.user_manager {
            return Err("user_manager".into());
        }
        Ok(user)
    }

    fn can_i_admin(self) -> Result<User, MissingPermission> {
        if self.is_none() {
            return Err("Not Logged In".into());
        }
        let user = self.unwrap();
        if !user.permissions.admin {
            return Err("admin".into());
        }
        Ok(user)
    }
}
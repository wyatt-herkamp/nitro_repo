use std::fmt::Debug;

use serde::{Deserialize, Serialize};
use crate::system::permissions::UserPermissions;


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserListResponse {
    pub id: i64,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserResponse {
    pub id: i64,
    pub name: String,
    pub username: String,
    pub email: String,
    pub permissions: UserPermissions,
    pub created: i64,
}






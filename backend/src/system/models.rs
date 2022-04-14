use std::fmt::Debug;

use serde::{Deserialize, Serialize};

use crate::schema::*;
use crate::system::permissions::UserPermissions;

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Insertable)]
#[table_name = "users"]
pub struct User {
    pub id: i64,
    pub name: String,
    pub username: String,
    pub email: String,
    #[serde(skip_serializing)]
    pub password: String,
    pub permissions: UserPermissions,
    pub created: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Queryable)]
pub struct UserListResponse {
    pub id: i64,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Queryable)]
pub struct UserResponse {
    pub id: i64,
    pub name: String,
    pub username: String,
    pub email: String,
    pub permissions: UserPermissions,
    pub created: i64,
}

impl User {
    pub fn set_password(&mut self, password: String) {
        self.password = password;
    }
}




#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Insertable)]
pub struct ForgotPassword {
    pub id: i64,
    pub user: i64,
    pub token: String,
    pub expiration: i64,
    pub created: i64,
}

// Represents a Session of an active user
#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Insertable)]
pub struct SessionToken {
    pub id: i64,
    pub user: i64,
    pub token: String,
    pub expiration: i64,
    pub created: i64,
}

// Unlike a SessionToken this is a token sent with the users username to be used as a password.
// If the user sets up Google Authentication with their account they will need to generate one of these to do deploys
#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Insertable)]
pub struct AuthToken {
    pub id: i64,
    pub user: i64,
    pub token: String,
    pub expiration: i64,
    pub created: i64,
}

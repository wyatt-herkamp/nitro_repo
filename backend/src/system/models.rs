use std::fmt::Debug;

use sea_orm::{FromQueryResult, JsonValue};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, FromQueryResult)]
pub struct UserListResponse {
    pub id: i64,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromQueryResult)]
pub struct UserResponse {
    pub id: i64,
    pub name: String,
    pub username: String,
    pub email: String,
    pub permissions: JsonValue,
    pub created: i64,
}

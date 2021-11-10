use serde::{Deserialize, Serialize};

use crate::schema::*;

use std::fmt::Debug;

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Insertable)]
#[table_name = "storages"]
pub struct Storage {
    pub id: i64,
    pub public_name: String,
    pub name: String,
    pub created: i64,
}
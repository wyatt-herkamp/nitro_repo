use std::fmt::Debug;

use serde::{Deserialize, Serialize};


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Storage {
    pub id: i64,
    pub public_name: String,
    pub name: String,
    pub created: i64,
}
pub type Storages = Vec<Storage>;
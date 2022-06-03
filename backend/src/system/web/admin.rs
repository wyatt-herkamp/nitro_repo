use serde::{Deserialize, Serialize};

// struct that derives Serialize and Deserialize contains the number of active storages, number of active repositories, and the number of active users.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SystemStatus {
    pub active_storages: usize,
    pub active_repositories: usize,
    pub active_users: usize,
}

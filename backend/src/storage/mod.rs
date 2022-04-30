use crate::storage::models::{Storage, StorageFile, StorageFileResponse};

use serde::{Serialize, Serializer};

use async_trait::async_trait;
use thiserror::Error;
use tokio::sync::RwLockReadGuard;

pub mod admin;
pub mod local_storage;
pub mod models;
pub mod multi;


pub static STORAGES_CONFIG: &str = "storages.nitro_repo";
pub static STORAGE_CONFIG: &str = "storage.nitro_repo";

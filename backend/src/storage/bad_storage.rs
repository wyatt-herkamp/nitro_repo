use crate::storage::error::StorageError;

use crate::storage::models::StorageStatus;
use crate::storage::StorageSaver;

/// This is a storages that is here to represent a storages that failed to load from the config stage
#[derive(Debug)]
pub struct BadStorage {
    pub factory: StorageSaver,
    pub status: StorageStatus,
}
impl BadStorage {
    pub fn create(factory: StorageSaver, error: StorageError) -> BadStorage {
        BadStorage {
            factory,
            status: StorageStatus::CreateError(error),
        }
    }
}

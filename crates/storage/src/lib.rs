#![allow(irrefutable_let_patterns)]
#![allow(unreachable_patterns)]
use std::future::Future;

pub use config::*;
pub use error::StorageError;
pub use fs::*;
use nr_core::storage::StoragePath;
use serde_json::Value;
use tracing::warn;
pub use uuid::Uuid;
mod config;
mod dyn_storage;
mod error;
mod fs;
pub use dyn_storage::*;
pub mod local;
pub trait Storage: Send + Sync {
    /// Unload the storages
    fn unload(&self) -> impl Future<Output = Result<(), StorageError>> + Send;

    fn storage_type(&self) -> BorrowedStorageTypeConfig<'_> {
        self.storage_config().config.clone()
    }
    fn storage_config(&self) -> BorrowedStorageConfig<'_>;

    /// Saves a File to a location
    /// Will overwrite any data found
    ///
    /// # Result
    /// Return the number of bytes written and if a new file was created
    async fn save_file(
        &self,
        repository: Uuid,
        file: FileContent,
        location: &StoragePath,
    ) -> Result<(usize, bool), StorageError>;
    /// Repository Meta files are files that are not listed and the repository controls the content. The content is stored as JSON
    async fn put_repository_meta(
        &self,
        repository: Uuid,
        location: &StoragePath,
        value: Value,
    ) -> Result<(), StorageError>;
    /// Repository Meta files are files that are not listed and the repository controls the content. The content is stored as JSON
    async fn get_repository_meta(
        &self,
        repository: Uuid,
        location: &StoragePath,
    ) -> Result<Option<Value>, StorageError>;
    /// Deletes a file at a given location
    async fn delete_file(
        &self,
        repository: Uuid,
        location: &StoragePath,
    ) -> Result<(), StorageError>;

    /// Returns Information about the file
    async fn get_file_information(
        &self,
        repository: Uuid,
        location: &StoragePath,
    ) -> Result<Option<StorageFileMeta>, StorageError>;

    /// Gets the File Information and Content
    ///
    /// range is ignored for directories
    /// range is the byte range to read from the file
    async fn open_file(
        &self,
        repository: Uuid,
        location: &StoragePath,
    ) -> Result<Option<StorageFile>, StorageError>;

    async fn validate_config_change(&self, config: StorageTypeConfig) -> Result<(), StorageError>;
}
pub trait StorageFactory {
    fn storage_name(&self) -> &'static str;

    //TODO fn storage_config_schema(&self) -> StorageTypeConfigSchema;
    async fn test_storage_config(&self, config: StorageTypeConfig) -> Result<(), StorageError>;

    async fn create_storage(&self, config: StorageConfig) -> Result<DynStorage, StorageError>;
}
#[cfg(test)]
mod tests {
    #[test]
    pub fn test_build() {
        println!("Test Build");
    }
}

#![allow(irrefutable_let_patterns)]
#![allow(unreachable_patterns)]
use std::future::Future;

pub use config::*;
pub use error::StorageError;
pub use fs::*;
use futures::future::BoxFuture;
use nr_core::storage::StoragePath;
use serde_json::Value;
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
    fn save_file(
        &self,
        repository: Uuid,
        file: FileContent,
        location: &StoragePath,
    ) -> impl Future<Output = Result<(usize, bool), StorageError>> + Send;
    /// Repository Meta files are files that are not listed and the repository controls the content. The content is stored as JSON
    fn put_repository_meta(
        &self,
        repository: Uuid,
        location: &StoragePath,
        value: Value,
    ) -> impl Future<Output = Result<(), StorageError>> + Send;
    /// Repository Meta files are files that are not listed and the repository controls the content. The content is stored as JSON
    fn get_repository_meta(
        &self,
        repository: Uuid,
        location: &StoragePath,
    ) -> impl Future<Output = Result<Option<Value>, StorageError>> + Send;
    /// Deletes a file at a given location
    fn delete_file(
        &self,
        repository: Uuid,
        location: &StoragePath,
    ) -> impl Future<Output = Result<(), StorageError>> + Send;

    /// Returns Information about the file
    fn get_file_information(
        &self,
        repository: Uuid,
        location: &StoragePath,
    ) -> impl Future<Output = Result<Option<StorageFileMeta>, StorageError>> + Send;

    /// Gets the File Information and Content
    ///
    /// range is ignored for directories
    /// range is the byte range to read from the file
    fn open_file(
        &self,
        repository: Uuid,
        location: &StoragePath,
    ) -> impl Future<Output = Result<Option<StorageFile>, StorageError>> + Send;

    fn validate_config_change(
        &self,
        config: StorageTypeConfig,
    ) -> impl Future<Output = Result<(), StorageError>> + Send;
}
pub trait StorageFactory: Send + Sync {
    fn storage_name(&self) -> &'static str;

    //TODO fn storage_config_schema(&self) -> StorageTypeConfigSchema;
    fn test_storage_config(
        &self,
        config: StorageTypeConfig,
    ) -> BoxFuture<'static, Result<(), StorageError>>;

    fn create_storage(
        &self,
        config: StorageConfig,
    ) -> BoxFuture<'static, Result<DynStorage, StorageError>>;
}
#[cfg(test)]
mod tests {
    #[test]
    pub fn test_build() {
        println!("Test Build");
    }
}

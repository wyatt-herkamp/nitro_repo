pub use config::*;
pub use error::StorageError;
pub use fs::*;
use serde_json::Value;
use tracing::warn;
pub use uuid::Uuid;
mod config;
mod error;
mod fs;
pub mod local;
pub trait Storage: Send + Sync {
    /// Unload the storages
    async fn unload(&mut self) -> Result<(), StorageError>;

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
        location: impl Into<StoragePath>,
    ) -> Result<(usize, bool), StorageError>;
    async fn put_file_meta(
        _repository: Uuid,
        _location: impl Into<StoragePath>,
        _key: impl Into<String>,
        _value: impl Into<Value>,
    ) -> Result<(), StorageError> {
        warn!("put_file_meta is not implemented for this storage");
        Ok(())
    }
    async fn get_file_meta(
        _repository: Uuid,
        _location: impl Into<StoragePath>,
        _key: impl Into<String>,
    ) -> Result<Option<Value>, StorageError> {
        warn!("get_file_meta is not implemented for this storage");
        Ok(None)
    }
    /// Deletes a file at a given location
    async fn delete_file(
        &self,
        repository: Uuid,
        location: impl Into<StoragePath>,
    ) -> Result<(), StorageError>;

    /// Returns Information about the file
    async fn get_file_information(
        &self,
        repository: Uuid,
        location: impl Into<StoragePath>,
    ) -> Result<Option<StorageFileMeta>, StorageError>;

    /// Gets the File Information and Content
    ///
    /// range is ignored for directories
    /// range is the byte range to read from the file
    async fn open_file(
        &self,
        repository: Uuid,
        location: impl Into<StoragePath>,
    ) -> Result<Option<StorageFile>, StorageError>;
}

#[cfg(test)]
mod tests {
    #[test]
    pub fn test_build() {
        println!("Test Build");
    }
}

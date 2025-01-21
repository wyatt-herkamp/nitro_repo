#![allow(irrefutable_let_patterns)]
#![allow(unreachable_patterns)]
use std::future::Future;
pub mod s3;
pub use config::*;
pub use error::StorageError;
pub use fs::*;
use futures::future::BoxFuture;
use meta::RepositoryMeta;
use nr_core::storage::StoragePath;
pub use uuid::Uuid;
mod config;
mod dyn_storage;
mod error;
mod fs;
pub use dyn_storage::*;
pub mod local;
pub mod meta;
#[cfg(test)]
pub(crate) mod testing;
pub(crate) mod utils;

pub trait Storage: Send + Sync {
    type Error: Into<StorageError> + std::error::Error + Send + Sync + 'static;
    /// Unload the storages
    fn unload(&self) -> impl Future<Output = Result<(), Self::Error>> + Send;
    fn storage_type_name(&self) -> &'static str;
    fn storage_type(&self) -> BorrowedStorageTypeConfig<'_> {
        self.storage_config().config
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
    ) -> impl Future<Output = Result<(usize, bool), Self::Error>> + Send;
    /// Repository Meta files are files that are not listed and the repository controls the content. The content is stored as JSON
    fn put_repository_meta(
        &self,
        repository: Uuid,
        location: &StoragePath,
        value: RepositoryMeta,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send;
    /// Repository Meta files are files that are not listed and the repository controls the content.
    ///
    /// Returns None if the file does not exist
    ///
    /// Returns Empty if the file exists but has no meta data
    fn get_repository_meta(
        &self,
        repository: Uuid,
        location: &StoragePath,
    ) -> impl Future<Output = Result<Option<RepositoryMeta>, Self::Error>> + Send;
    /// Deletes a file at a given location
    fn delete_file(
        &self,
        repository: Uuid,
        location: &StoragePath,
    ) -> impl Future<Output = Result<bool, Self::Error>> + Send;

    /// Returns Information about the file
    fn get_file_information(
        &self,
        repository: Uuid,
        location: &StoragePath,
    ) -> impl Future<Output = Result<Option<StorageFileMeta<FileType>>, Self::Error>> + Send;

    /// Gets the File Information and Content
    ///
    /// range is ignored for directories
    /// range is the byte range to read from the file
    fn open_file(
        &self,
        repository: Uuid,
        location: &StoragePath,
    ) -> impl Future<Output = Result<Option<StorageFile>, Self::Error>> + Send;

    fn validate_config_change(
        &self,
        config: StorageTypeConfig,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send;

    fn file_exists(
        &self,
        repository: Uuid,
        location: &StoragePath,
    ) -> impl Future<Output = Result<bool, Self::Error>> + Send;
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

pub trait StaticStorageFactory: StorageFactory {
    type StorageType: Storage;
    type ConfigType: StorageTypeConfigTrait;
    type Error: Into<StorageError>;
    fn storage_type_name() -> &'static str
    where
        Self: Sized;

    async fn test_storage_config(config: StorageTypeConfig) -> Result<(), Self::Error>;

    async fn create_storage(
        inner: StorageConfigInner,
        type_config: Self::ConfigType,
    ) -> Result<Self::StorageType, Self::Error>;

    async fn create_storage_from_config(
        config: StorageConfig,
    ) -> Result<Self::StorageType, Self::Error>
    where
        Self::Error: From<InvalidConfigType>,
    {
        let inner = config.storage_config;
        let type_config = Self::ConfigType::from_type_config(config.type_config.into())?;
        <Self as StaticStorageFactory>::create_storage(inner, type_config).await
    }
}
#[cfg(test)]
mod tests {
    #[test]
    pub fn test_build() {
        println!("Test Build");
    }
}

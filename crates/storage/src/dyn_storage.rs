use nr_core::storage::StoragePath;
use uuid::Uuid;

use crate::{
    FileContent, FileType, Storage, StorageError, StorageFactory, StorageTypeConfig,
    fs_v2::{FileSystemV2Factory, FileSystemV2Storage},
    local::{LocalStorage, LocalStorageFactory},
    meta::RepositoryMeta,
    s3::{S3Storage, S3StorageFactory},
    streaming::DynDirectoryListStream,
};

/// Generates the dynamic-dispatch enum over every storage backend.
///
/// [Storage] uses `impl Future` in return position, which makes it not object safe, so dispatch
/// cannot go through `Box<dyn Storage>`. This enum stands in for that — but writing it by hand
/// meant one match arm per method per backend, so adding a backend meant eleven near-identical
/// arms that all had to be right. The macro is the same trick [crate::config::storage_type_config]
/// already uses for the config registry.
macro_rules! dyn_storage {
    (
        $(
            $variant:ident($storage:ty)
        ),* $(,)?
    ) => {
        #[derive(Debug, Clone)]
        pub enum DynStorage {
            $(
                $variant($storage),
            )*
        }

        $(
            impl From<$storage> for DynStorage {
                fn from(storage: $storage) -> Self {
                    DynStorage::$variant(storage)
                }
            }
        )*

        impl Storage for DynStorage {
            type Error = StorageError;
            type DirectoryStream = DynDirectoryListStream;

            fn storage_type_name(&self) -> &'static str {
                match self {
                    $( DynStorage::$variant(storage) => storage.storage_type_name(), )*
                }
            }

            fn storage_config(&self) -> crate::BorrowedStorageConfig<'_> {
                match self {
                    $( DynStorage::$variant(storage) => storage.storage_config(), )*
                }
            }

            async fn unload(&self) -> Result<(), StorageError> {
                match self {
                    $( DynStorage::$variant(storage) => storage.unload().await.map_err(Into::into), )*
                }
            }

            async fn save_file(
                &self,
                repository: Uuid,
                file: FileContent,
                location: &StoragePath,
            ) -> Result<(usize, bool), StorageError> {
                match self {
                    $(
                        DynStorage::$variant(storage) => storage
                            .save_file(repository, file, location)
                            .await
                            .map_err(Into::into),
                    )*
                }
            }

            async fn delete_file(
                &self,
                repository: Uuid,
                location: &StoragePath,
            ) -> Result<bool, StorageError> {
                match self {
                    $(
                        DynStorage::$variant(storage) => storage
                            .delete_file(repository, location)
                            .await
                            .map_err(Into::into),
                    )*
                }
            }

            async fn get_file_information(
                &self,
                repository: Uuid,
                location: &StoragePath,
            ) -> Result<Option<crate::StorageFileMeta<FileType>>, StorageError> {
                match self {
                    $(
                        DynStorage::$variant(storage) => storage
                            .get_file_information(repository, location)
                            .await
                            .map_err(Into::into),
                    )*
                }
            }

            async fn open_file(
                &self,
                repository: Uuid,
                location: &StoragePath,
            ) -> Result<Option<crate::StorageFile>, StorageError> {
                match self {
                    $(
                        DynStorage::$variant(storage) => storage
                            .open_file(repository, location)
                            .await
                            .map_err(Into::into),
                    )*
                }
            }

            async fn validate_config_change(
                &self,
                config: StorageTypeConfig,
            ) -> Result<(), StorageError> {
                match self {
                    $(
                        DynStorage::$variant(storage) => storage
                            .validate_config_change(config)
                            .await
                            .map_err(Into::into),
                    )*
                }
            }

            async fn put_repository_meta(
                &self,
                repository: Uuid,
                location: &StoragePath,
                value: RepositoryMeta,
            ) -> Result<(), StorageError> {
                match self {
                    $(
                        DynStorage::$variant(storage) => storage
                            .put_repository_meta(repository, location, value)
                            .await
                            .map_err(Into::into),
                    )*
                }
            }

            async fn get_repository_meta(
                &self,
                repository: Uuid,
                location: &StoragePath,
            ) -> Result<Option<RepositoryMeta>, StorageError> {
                match self {
                    $(
                        DynStorage::$variant(storage) => storage
                            .get_repository_meta(repository, location)
                            .await
                            .map_err(Into::into),
                    )*
                }
            }

            async fn file_exists(
                &self,
                repository: Uuid,
                location: &StoragePath,
            ) -> Result<bool, StorageError> {
                match self {
                    $(
                        DynStorage::$variant(storage) => storage
                            .file_exists(repository, location)
                            .await
                            .map_err(Into::into),
                    )*
                }
            }

            async fn stream_directory(
                &self,
                repository: Uuid,
                location: &StoragePath,
            ) -> Result<Option<Self::DirectoryStream>, Self::Error> {
                match self {
                    $(
                        DynStorage::$variant(storage) => storage
                            .stream_directory(repository, location)
                            .await
                            .map(|stream| stream.map(DynDirectoryListStream::new))
                            .map_err(Into::into),
                    )*
                }
            }
        }
    };
}

dyn_storage! {
    Local(LocalStorage),
    S3(S3Storage),
    FileSystemV2(FileSystemV2Storage),
}

/// Every backend the server can load, in the order they are offered.
pub static STORAGE_FACTORIES: &[&dyn StorageFactory] = &[
    &LocalStorageFactory,
    &S3StorageFactory,
    &FileSystemV2Factory,
];

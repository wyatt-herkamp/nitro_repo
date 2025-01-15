use std::{
    fs::{self},
    io::{self, ErrorKind},
    ops::Deref,
    path::PathBuf,
    sync::Arc,
};

pub mod error;
use error::LocalStorageError;
use nr_core::storage::StoragePath;
use serde::{Deserialize, Serialize};
use tokio::task::JoinSet;
use tracing::{debug, error, info, instrument, trace, warn};
use utils::new_type_arc_type;

use crate::*;
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LocalConfig {
    pub path: PathBuf,
}
impl utoipa::__dev::ComposeSchema for LocalConfig {
    fn compose(
        _generics: Vec<utoipa::openapi::RefOr<utoipa::openapi::schema::Schema>>,
    ) -> utoipa::openapi::RefOr<utoipa::openapi::schema::Schema> {
        utoipa::openapi::ObjectBuilder::new()
            .property(
                "path",
                utoipa::openapi::ObjectBuilder::new().schema_type(
                    utoipa::openapi::schema::SchemaType::new(utoipa::openapi::schema::Type::String),
                ),
            )
            .required("path")
            .into()
    }
}
impl utoipa::ToSchema for LocalConfig {
    fn name() -> std::borrow::Cow<'static, str> {
        std::borrow::Cow::Borrowed("LocalConfig")
    }
}
impl utoipa::__dev::SchemaReferences for LocalConfig {
    fn schemas(
        schemas: &mut Vec<(
            String,
            utoipa::openapi::RefOr<utoipa::openapi::schema::Schema>,
        )>,
    ) {
        schemas.extend([]);
    }
}
#[derive(Debug)]
pub struct LocalStorageInner {
    pub config: LocalConfig,
    pub storage_config: StorageConfigInner,
}
impl LocalStorageInner {}
#[derive(Debug, Clone)]
pub struct LocalStorage(Arc<LocalStorageInner>);
new_type_arc_type!(LocalStorage(LocalStorageInner));

impl LocalStorageInner {
    /// Get the path for a file to be created
    ///
    /// # Returns
    /// ## Ok
    /// - First value is the path to the file
    /// - Second is the directory that the file is in
    pub fn get_path_for_creation(
        &self,
        repository: Uuid,
        location: &StoragePath,
    ) -> Result<(PathBuf, PathBuf), PathCollisionError> {
        let mut path = self.config.path.join(repository.to_string());
        let mut parent_directory = path.clone();
        let mut conflicting_path = StoragePath::default();
        let mut iter = location.clone().into_iter().peekable();
        while let Some(part) = iter.next() {
            if iter.peek().is_none() {
                parent_directory = path.clone();
            }
            path = path.join(part.as_ref());
            conflicting_path.push_mut(part.as_ref());
            if path.exists() {
                if path.is_file() {
                    return Err(PathCollisionError {
                        path: location.clone(),
                        conflicts_with: conflicting_path,
                    });
                }
            }
        }
        Ok((path, parent_directory))
    }
    #[instrument(skip(location))]
    pub fn get_path(&self, repository: &Uuid, location: &StoragePath) -> PathBuf {
        let location: PathBuf = location.into();
        let path = self.config.path.join(repository.to_string());
        path.join(location)
    }

    #[instrument]
    pub fn open_file(&self, path: PathBuf) -> Result<StorageFile, LocalStorageError> {
        let meta = StorageFileMeta::read_from_file(&path)?;
        let file = fs::File::open(&path)?;
        Ok(StorageFile::File {
            meta,
            content: StorageFileReader::from(file),
        })
    }
    #[instrument(skip(path))]
    pub async fn open_folder(&self, path: PathBuf) -> Result<StorageFile, LocalStorageError> {
        let mut set = JoinSet::<Result<StorageFileMeta<FileType>, LocalStorageError>>::new();

        for entry in fs::read_dir(&path)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() && is_hidden_file(&path) {
                trace!(?path, "Skipping Meta File");
                // Check if file is a meta file
                continue;
            }
            set.spawn_blocking(move || {
                let meta = StorageFileMeta::read_from_path(&path)?;
                Ok(meta)
            });
        }
        let meta = StorageFileMeta::read_from_directory(path)?;

        let mut files = vec![];
        while let Some(res) = set.join_next().await {
            let idx = res.unwrap();
            files.push(idx?);
        }

        Ok(StorageFile::Directory { meta, files })
    }
}
impl LocalStorage {
    pub fn run_post_save_file(self, path: PathBuf) -> Result<(), LocalStorageError> {
        tokio::task::spawn_blocking(move || {
            match FileMeta::create_meta_or_update(&path) {
                Ok(()) => {
                    info!(?path, "Meta File Created or Updated");
                }
                Err(err) => {
                    error!(?err, "Unable to create or update meta file");
                }
            };
        });
        Ok(())
    }
}
impl Storage for LocalStorage {
    type Error = LocalStorageError;
    fn storage_config(&self) -> BorrowedStorageConfig<'_> {
        BorrowedStorageConfig {
            storage_config: &self.storage_config,
            config: BorrowedStorageTypeConfig::Local(&self.config),
        }
    }
    #[instrument(name = "local_storage_save")]
    async fn save_file(
        &self,
        repository: Uuid,
        content: FileContent,
        location: &StoragePath,
    ) -> Result<(usize, bool), LocalStorageError> {
        let (path, parent_directory) = self.0.get_path_for_creation(repository, location)?;
        if !parent_directory.exists() {
            trace!("Creating Parent Directory");
            fs::create_dir_all(parent_directory)?;
        } else if parent_directory.is_file() {
            warn!(?parent_directory, "Parent Directory is a file");
            return Err(
                io::Error::new(io::ErrorKind::InvalidInput, "Parent Directory is a file").into(),
            );
        }
        let new_file = !path.exists();
        debug!(?path, "Saving File");
        let mut file = fs::File::create(&path)?;
        let bytes_written = content.write_to(&mut file)?;
        if !is_hidden_file(&path) {
            // Don't run post save file for meta files
            self.clone().run_post_save_file(path)?;
        }
        Ok((bytes_written, new_file))
    }
    #[instrument(name = "local_storage_delete")]
    async fn delete_file(
        &self,
        repository: Uuid,
        location: &StoragePath,
    ) -> Result<bool, LocalStorageError> {
        let path = self.get_path(&repository, location);
        if !path.exists() {
            debug!(?path, "File does not exist");
            return Ok(false);
        }
        if path.is_dir() {
            info!(?path, "Deleting Directory");
            fs::remove_dir_all(path)?;
        } else {
            fs::remove_file(&path)?;
            FileMeta::delete_local(path)?;
        }
        Ok(true)
    }
    #[instrument(name = "local_storage_get_info")]
    async fn get_file_information(
        &self,
        repository: Uuid,
        location: &StoragePath,
    ) -> Result<Option<StorageFileMeta<FileType>>, LocalStorageError> {
        let path = self.get_path(&repository, location);

        if !path.exists() {
            debug!(?path, "File does not exist");
            return Ok(None);
        }
        let meta = StorageFileMeta::read_from_path(path)?;
        Ok(Some(meta))
    }
    #[instrument(name = "local_storage_open_file")]
    async fn open_file(
        &self,
        repository: Uuid,
        location: &StoragePath,
    ) -> Result<Option<StorageFile>, LocalStorageError> {
        let path = self.get_path(&repository, location);
        if !path.exists() {
            debug!(?path, "File does not exist");
            return Ok(None);
        }
        let file = if path.is_dir() {
            self.open_folder(path).await?
        } else {
            self.0.open_file(path)?
        };
        Ok(Some(file))
    }
    async fn unload(&self) -> Result<(), LocalStorageError> {
        info!(?self, "Unloading Local Storage");
        // TODO: Implement Unload
        Ok(())
    }

    async fn validate_config_change(
        &self,
        config: StorageTypeConfig,
    ) -> Result<(), LocalStorageError> {
        let config = LocalConfig::from_type_config(config)?;
        if self.config.path != config.path {
            return Err(LocalStorageError::PathCannotBeChanged);
        }
        Ok(())
    }
    async fn get_repository_meta(
        &self,
        repository: Uuid,
        location: &StoragePath,
    ) -> Result<Option<RepositoryMeta>, LocalStorageError> {
        let path = self.get_path(&repository, location);
        if !path.exists() {
            return Ok(None);
        }
        let meta = FileMeta::get_or_create_local(&path)?;
        Ok(Some(meta.repository_meta))
    }
    async fn put_repository_meta(
        &self,
        repository: Uuid,
        location: &StoragePath,
        value: RepositoryMeta,
    ) -> Result<(), LocalStorageError> {
        let path = self.get_path(&repository, location);

        if !path.exists() {
            return Err(LocalStorageError::IOError(io::Error::new(
                ErrorKind::NotFound,
                "File not found",
            )));
        }

        FileMeta::set_repository_meta(path, value)?;
        Ok(())
    }
    async fn file_exists(
        &self,
        repository: Uuid,
        location: &StoragePath,
    ) -> Result<bool, LocalStorageError> {
        let path = self.get_path(&repository, location);
        Ok(path.exists())
    }
}
#[derive(Debug, Default)]
pub struct LocalStorageFactory;
impl StaticStorageFactory for LocalStorageFactory {
    type StorageType = LocalStorage;

    type ConfigType = LocalConfig;

    type Error = LocalStorageError;

    fn storage_type_name() -> &'static str
    where
        Self: Sized,
    {
        "Local"
    }

    async fn test_storage_config(_: StorageTypeConfig) -> Result<(), LocalStorageError> {
        Ok(())
    }

    async fn create_storage(
        inner: StorageConfigInner,
        type_config: Self::ConfigType,
    ) -> Result<Self::StorageType, LocalStorageError> {
        if !type_config.path.exists() {
            fs::create_dir_all(&type_config.path)?;
        }
        let inner = LocalStorageInner {
            config: type_config,
            storage_config: inner,
        };
        let storage = LocalStorage::from(inner);

        Ok(storage)
    }
}
impl StorageFactory for LocalStorageFactory {
    fn storage_name(&self) -> &'static str {
        Self::storage_type_name()
    }

    fn test_storage_config(
        &self,
        _config: StorageTypeConfig,
    ) -> BoxFuture<'static, Result<(), StorageError>> {
        Box::pin(async move { Ok(()) })
    }

    fn create_storage(
        &self,
        config: StorageConfig,
    ) -> BoxFuture<'static, Result<DynStorage, StorageError>> {
        Box::pin(async move {
            <Self as StaticStorageFactory>::create_storage_from_config(config)
                .await
                .map(|storage| DynStorage::Local(storage))
                .map_err(Into::into)
        })
    }
}

#[cfg(test)]
mod tests {
    use tracing::warn;

    use crate::{
        local::LocalStorageFactory, testing::storage::TestingStorage, StaticStorageFactory,
    };

    #[tokio::test]
    pub async fn generic_test() -> anyhow::Result<()> {
        let Some(config) = crate::testing::start_storage_test("Local")? else {
            warn!("Local Storage Test Skipped");
            return Ok(());
        };
        let local_storage =
            <LocalStorageFactory as StaticStorageFactory>::create_storage_from_config(config)
                .await?;
        let testing_storage = TestingStorage::new(local_storage);
        crate::testing::tests::full_test(testing_storage).await?;

        Ok(())
    }
}

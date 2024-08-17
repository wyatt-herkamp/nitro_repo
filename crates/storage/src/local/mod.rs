use std::{
    fs::{self},
    io::{self},
    ops::Deref,
    path::PathBuf,
    sync::Arc,
};

use nr_core::storage::StoragePath;
use serde::{Deserialize, Serialize};
use tracing::{debug, info, instrument, span, trace, warn, Level};
use utils::PathUtils;
use utoipa::ToSchema;

use crate::*;
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct LocalConfig {
    pub path: PathBuf,
}
#[derive(Debug)]
pub struct LocalStorageInner {
    pub config: LocalConfig,
    pub storage_config: StorageConfigInner,
}
impl LocalStorageInner {}
#[derive(Debug, Clone)]
pub struct LocalStorage(Arc<LocalStorageInner>);
impl Deref for LocalStorage {
    type Target = LocalStorageInner;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl LocalStorageInner {
    #[instrument(skip(location))]
    pub fn get_path(&self, repository: &Uuid, location: &StoragePath) -> PathBuf {
        let location: PathBuf = location.into();
        let path = self.config.path.join(repository.to_string());
        path.join(location)
    }
    pub fn get_repository_meta_path(
        &self,
        repository: &Uuid,
        location: &StoragePath,
    ) -> Result<PathBuf, StorageError> {
        let path = self.get_path(&repository, location);
        let path = if path.is_dir() {
            let Some(folder_name) = path.file_name() else {
                return Err(StorageError::IOError(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "Path is a directory but has no file name",
                )));
            };
            path.parent_or_err()?
                .join(folder_name)
                .add_extension("nr-repository-meta")?
        } else {
            path.add_extension("nr-repository-meta")?
        };
        Ok(path)
    }

    #[instrument]
    pub fn open_file(&self, path: PathBuf) -> Result<StorageFile, StorageError> {
        let meta = StorageFileMeta::new_from_file(&path)?;
        let file = fs::File::open(&path)?;
        Ok(StorageFile::File {
            meta,
            content: StorageFileReader::from(file),
        })
    }
    #[instrument]
    pub fn open_folder(&self, path: PathBuf) -> Result<StorageFile, StorageError> {
        let mut files = vec![];
        for entry in fs::read_dir(&path)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() && path.extension().unwrap_or_default() == "nr-meta" {
                trace!(?path, "Skipping Meta File");
                // Check if file is a meta file
                continue;
            }
            let meta = StorageFileMeta::new_from_file(path)?;
            files.push(meta);
        }
        let meta = StorageFileMeta::new_from_file(path)?;
        Ok(StorageFile::Directory { meta, files })
    }
}
impl LocalStorage {
    pub fn run_post_save_file(self, path: PathBuf) -> Result<(), StorageError> {
        info!(?path, "Running Post Save File");
        Ok(())
    }
}
impl Storage for LocalStorage {
    fn storage_config(&self) -> BorrowedStorageConfig<'_> {
        BorrowedStorageConfig {
            storage_config: &self.storage_config,
            config: BorrowedStorageTypeConfig::Local(&self.config),
        }
    }
    async fn save_file(
        &self,
        repository: Uuid,
        content: FileContent,
        location: &StoragePath,
    ) -> Result<(usize, bool), StorageError> {
        let span = span!(Level::DEBUG, "local_storage_save", "repository" = ?repository, "location" = ?location, "storage_id" = ?self.storage_config.storage_id);
        let _enter = span.enter();
        let path = self.get_path(&repository, location);
        let parent_directory = path.parent_or_err()?;
        let new_file = !path.exists();
        if !parent_directory.exists() {
            trace!("Creating Parent Directory");
            fs::create_dir_all(parent_directory)?;
        } else if parent_directory.is_file() {
            warn!(?parent_directory, "Parent Directory is a file");
            return Err(
                io::Error::new(io::ErrorKind::InvalidInput, "Parent Directory is a file").into(),
            );
        }

        let mut file = fs::File::create(&path)?;
        let bytes_written = content.write_to(&mut file)?;
        if path.extension().unwrap_or_default() != "nr-meta" {
            // Don't run post save file for meta files
            self.clone().run_post_save_file(path)?;
        }
        Ok((bytes_written, new_file))
    }
    async fn delete_file(
        &self,
        repository: Uuid,
        location: &StoragePath,
    ) -> Result<(), StorageError> {
        let span = span!(Level::DEBUG, "local_storage_delete", "repository" = ?repository, "location" = ?location, "storage_id" = ?self.storage_config.storage_id);
        let _enter = span.enter();
        let path = self.get_path(&repository, location);
        if path.is_dir() {
            info!(?path, "Deleting Directory");
            fs::remove_dir_all(path)?;
        } else {
            fs::remove_file(&path)?;
            FileMeta::delete_local(path)?;
        }
        Ok(())
    }
    async fn get_file_information(
        &self,
        repository: Uuid,
        location: &StoragePath,
    ) -> Result<Option<StorageFileMeta>, StorageError> {
        let span = span!(Level::DEBUG, "local_storage_get_info", "repository" = ?repository, "location" = ?location, "storage_id" = ?self.storage_config.storage_id);
        let _enter = span.enter();
        let path = self.get_path(&repository, location);

        if !path.exists() {
            debug!(?path, "File does not exist");
            return Ok(None);
        }
        let meta = StorageFileMeta::new_from_file(path)?;
        Ok(Some(meta))
    }
    async fn open_file(
        &self,
        repository: Uuid,
        location: &StoragePath,
    ) -> Result<Option<StorageFile>, StorageError> {
        let span = span!(Level::DEBUG, "local_storage_open_file", "repository" = ?repository, "location" = ?location, "storage_id" = ?self.storage_config.storage_id);
        let _enter = span.enter();
        let path = self.get_path(&repository, location);
        if !path.exists() {
            debug!(?path, "File does not exist");
            return Ok(None);
        }
        let file = if path.is_dir() {
            self.open_folder(path)?
        } else {
            self.0.open_file(path)?
        };
        Ok(Some(file))
    }
    fn unload(&self) -> impl std::future::Future<Output = Result<(), StorageError>> + Send {
        info!(?self, "Unloading Local Storage");
        // TODO: Implement Unload
        async { Ok(()) }
    }

    async fn validate_config_change(&self, config: StorageTypeConfig) -> Result<(), StorageError> {
        let StorageTypeConfig::Local(config) = config else {
            return Err(StorageError::InvalidConfigType("Local"));
        };
        if self.config.path != config.path {
            return Err(StorageError::ConfigError("The path cannot be changed"));
        }
        Ok(())
    }
    async fn get_repository_meta(
        &self,
        repository: Uuid,
        location: &StoragePath,
    ) -> Result<Option<Value>, StorageError> {
        let path = self.get_repository_meta_path(&repository, location)?;
        if !path.exists() {
            return Ok(None);
        }
        let file = fs::File::open(&path)?;
        let value: Value = serde_json::from_reader(file)?;
        Ok(Some(value))
    }
    async fn put_repository_meta(
        &self,
        repository: Uuid,
        location: &StoragePath,
        value: Value,
    ) -> Result<(), StorageError> {
        let path = self.get_repository_meta_path(&repository, location)?;
        if path.exists() {
            let path_backup = path.add_extension("bak")?;
            fs::rename(&path, &path_backup)?;
        }

        let parent = path.parent_or_err()?;
        if !parent.exists() {
            fs::create_dir_all(parent)?;
        }

        let file = fs::File::create(&path)?;
        serde_json::to_writer(file, &value)?;
        Ok(())
    }
}
#[derive(Debug, Default)]
pub struct LocalStorageFactory;
impl StorageFactory for LocalStorageFactory {
    fn storage_name(&self) -> &'static str {
        "Local"
    }

    fn test_storage_config(
        &self,
        config: StorageTypeConfig,
    ) -> BoxFuture<'static, Result<(), StorageError>> {
        Box::pin(async move { Ok(()) })
    }

    fn create_storage(
        &self,
        config: StorageConfig,
    ) -> BoxFuture<'static, Result<DynStorage, StorageError>> {
        Box::pin(async move {
            let local_config = match config.type_config {
                StorageTypeConfig::Local(config) => config,
                _ => return Err(StorageError::InvalidConfigType("Local")),
            };
            if !local_config.path.exists() {
                fs::create_dir_all(&local_config.path)?;
            }
            let local = LocalStorageInner {
                config: local_config,
                storage_config: config.storage_config,
            };
            Ok(DynStorage::Local(LocalStorage(Arc::new(local))))
        })
    }
}

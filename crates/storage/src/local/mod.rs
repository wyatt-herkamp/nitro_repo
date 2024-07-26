use std::{
    fs::{self},
    io::{self},
    ops::Deref,
    path::PathBuf,
    sync::Arc,
};

use serde::{Deserialize, Serialize};
use tracing::{debug, info, instrument, trace, warn};
use utils::PathUtils;

use crate::*;
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
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
    pub fn get_path(&self, repository: &Uuid, location: impl Into<StoragePath>) -> PathBuf {
        let location: PathBuf = location.into().into();
        let path = self.config.path.join(repository.to_string());
        path.join(location)
    }
    #[instrument]
    pub fn open_file(&self, path: PathBuf) -> Result<StorageFile, StorageError> {
        let meta = StorageFileMeta::new_from_file(&path)?;
        let file = fs::File::open(&path)?;
        Ok(StorageFile::File {
            meta,
            content: StorageFileReader::File(file),
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
    #[instrument(skip(content, location))]
    async fn save_file(
        &self,
        repository: Uuid,
        content: FileContent,
        location: impl Into<StoragePath>,
    ) -> Result<(usize, bool), StorageError> {
        let path = self.get_path(&repository, location);
        info!(?path, "Saving File");

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
    #[instrument(skip(location))]
    async fn delete_file(
        &self,
        repository: Uuid,
        location: impl Into<StoragePath>,
    ) -> Result<(), StorageError> {
        let path = self.get_path(&repository, location);
        info!(?path, "Deleting File");
        if path.is_dir() {
            info!(?path, "Deleting Directory");
            fs::remove_dir_all(path)?;
        } else {
            fs::remove_file(&path)?;
            FileMeta::delete_local(path)?;
        }
        Ok(())
    }
    #[instrument(skip(location))]
    async fn get_file_information(
        &self,
        repository: Uuid,
        location: impl Into<StoragePath>,
    ) -> Result<Option<StorageFileMeta>, StorageError> {
        let path = self.get_path(&repository, location);
        if !path.exists() {
            debug!(?path, "File does not exist");
            return Ok(None);
        }
        let meta = StorageFileMeta::new_from_file(path)?;
        Ok(Some(meta))
    }
    #[instrument(skip(location))]
    async fn open_file(
        &self,
        repository: Uuid,
        location: impl Into<StoragePath>,
    ) -> Result<Option<StorageFile>, StorageError> {
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
    #[instrument]
    async fn unload(&mut self) -> Result<(), StorageError> {
        info!(?self, "Unloading Local Storage");
        // TODO: Implement Unload
        Ok(())
    }
}

use std::{
    fs::{self},
    io::{self},
    ops::Deref,
    path::PathBuf,
    sync::Arc,
};

use nr_core::storage::StoragePath;
use serde::{Deserialize, Serialize};
use tokio::task::JoinSet;
use tracing::{debug, error, info, instrument, trace, warn};
use utils::PathUtils;

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
        let path = self.get_path(repository, location);
        let path = if path.is_dir() {
            let Some(folder_name) = path.file_name() else {
                return Err(StorageError::IOError(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "Path is a directory but has no file name",
                )));
            };
            path.parent_or_err()?
                .join(folder_name)
                .add_extension(NITRO_REPO_REPOSITORY_META_EXTENSION)?
        } else {
            path.add_extension(NITRO_REPO_REPOSITORY_META_EXTENSION)?
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
    #[instrument(skip(path))]
    pub async fn open_folder(&self, path: PathBuf) -> Result<StorageFile, StorageError> {
        let mut set = JoinSet::<Result<StorageFileMeta, StorageError>>::new();

        for entry in fs::read_dir(&path)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() && is_hidden_file(&path) {
                trace!(?path, "Skipping Meta File");
                // Check if file is a meta file
                continue;
            }
            set.spawn_blocking(move || {
                let meta = StorageFileMeta::new_from_file(&path)?;
                Ok(meta)
            });
        }
        let meta = StorageFileMeta::new_from_file(path)?;

        let mut files = vec![];
        while let Some(res) = set.join_next().await {
            let idx = res.unwrap();
            files.push(idx?);
        }

        Ok(StorageFile::Directory { meta, files })
    }
}
impl LocalStorage {
    pub fn run_post_save_file(self, path: PathBuf) -> Result<(), StorageError> {
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
    ) -> Result<(usize, bool), StorageError> {
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
    ) -> Result<(), StorageError> {
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
    #[instrument(name = "local_storage_get_info")]
    async fn get_file_information(
        &self,
        repository: Uuid,
        location: &StoragePath,
    ) -> Result<Option<StorageFileMeta>, StorageError> {
        let path = self.get_path(&repository, location);

        if !path.exists() {
            debug!(?path, "File does not exist");
            return Ok(None);
        }
        let meta = StorageFileMeta::new_from_file(path)?;
        Ok(Some(meta))
    }
    #[instrument(name = "local_storage_open_file")]
    async fn open_file(
        &self,
        repository: Uuid,
        location: &StoragePath,
    ) -> Result<Option<StorageFile>, StorageError> {
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
        _config: StorageTypeConfig,
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

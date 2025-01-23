use std::{
    fs::{self},
    io::{self, ErrorKind},
    ops::Deref,
    path::PathBuf,
    sync::Arc,
};

pub mod error;
use error::LocalStorageError;
use futures::FutureExt;
use nr_core::storage::StoragePath;
use serde::{Deserialize, Serialize};
use tokio::{
    sync::{Mutex, Notify},
    task::JoinSet,
};
use tracing::{
    debug, debug_span, error, event,
    field::{debug, Empty},
    info, info_span, instrument, trace, warn, Level, Span,
};
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
fn meta_update_task(
    mut shutdown: tokio::sync::oneshot::Receiver<()>,
    mut receiver: tokio::sync::mpsc::Receiver<PathBuf>,
) {
    tokio::task::spawn(async move {
        loop {
            tokio::select! {
                _ = &mut shutdown => {
                    break;
                }
                path = (&mut receiver).recv() => {
                    let Some(path) = path else{
                        break
                    };
                    let span = info_span!(
                        "Meta Update Task",
                        path = debug(&path),
                        otel.status_code = Empty,
                        otel.exception = Empty,
                    );
                    let _guard = span.enter();
                    if !path.exists(){
                        warn!("Path does not exist");
                        continue;
                    }
                    match  LocationMeta::create_meta_or_update(&path){
                        Ok(ok) => {
                            info!(?ok, "Updated Meta");
                            span.record("otel.status_code", "OK");
                        }
                        Err(err) => {
                            span.record("otel.status_code", "ERROR");
                            event!(Level::ERROR, ?err, "Error Updating Meta");
                        }
                  }
                }
            }
        }
        receiver.close();
    });
}

#[derive(Debug)]
pub struct LocalStorageInner {
    pub config: LocalConfig,
    pub storage_config: StorageConfigInner,
    pub shutdown_signal: Mutex<Option<tokio::sync::oneshot::Sender<()>>>,
    pub meta_update_sender: tokio::sync::mpsc::Sender<PathBuf>,
}
impl LocalStorageInner {}
#[derive(Debug, Clone)]
pub struct LocalStorage(Arc<LocalStorageInner>);
new_type_arc_type!(LocalStorage(LocalStorageInner));
struct CreatePath {
    path: PathBuf,
    parent_directory: PathBuf,
    /// The point at which the new directory starts
    ///
    /// If None, then the directory already exists
    new_directory_start: Option<PathBuf>,
}
impl LocalStorageInner {
    /// Get the path for a file to be created

    #[instrument(level = "debug")]
    fn get_path_for_creation(
        &self,
        repository: Uuid,
        location: &StoragePath,
    ) -> Result<CreatePath, LocalStorageError> {
        let mut path = self.config.path.join(repository.to_string());
        let mut parent_directory = path.clone();
        let mut new_directory_start = None;
        let mut conflicting_path = StoragePath::default();
        let mut iter = location.clone().into_iter().peekable();
        while let Some(part) = iter.next() {
            if iter.peek().is_none() {
                debug!(?part, "Last Part of Path");
                parent_directory = path.clone();
            }
            path = path.join(part.as_ref());
            conflicting_path.push_mut(part.as_ref());
            trace!(?path, ?conflicting_path, "Checking Path");
            if new_directory_start.is_some() {
                continue;
            }
            let metadata = match path.metadata() {
                // If the current path is a directory then we can continue as it can have a file inside it
                Ok(ok) if ok.is_dir() => {
                    continue;
                }
                Ok(ok) => ok,
                Err(err) if err.kind() == ErrorKind::NotFound => {
                    // Only Log this in debug mode or testing
                    #[cfg(any(debug_assertions, test))]
                    if tracing::enabled!(tracing::Level::TRACE) {
                        trace!(?path, "Path does not exist");
                    }
                    new_directory_start = Some(path.clone());
                    continue;
                }
                Err(err) => return Err(LocalStorageError::IOError(err)),
            };

            // If the current path is a file and we have more parts to add then we have a collision
            // Because you can't have a file inside a file
            if metadata.is_file() && iter.peek().is_some() {
                warn!(?path, "Path is a file");
                return Err(PathCollisionError {
                    path: location.clone(),
                    conflicts_with: conflicting_path,
                }
                .into());
            }
        }
        Ok(CreatePath {
            path,
            parent_directory,
            new_directory_start,
        })
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
    #[instrument(skip(path), fields(entries.read, entries.skipped))]
    pub async fn open_folder(&self, path: PathBuf) -> Result<StorageFile, LocalStorageError> {
        let mut set = JoinSet::<Result<StorageFileMeta<FileType>, LocalStorageError>>::new();
        let current_span = Span::current();
        let mut files_read = 0;
        let mut files_skipped = 0;
        let mut read_dir = tokio::fs::read_dir(&path).await?;
        while let Some(entry) = read_dir.next_entry().await.transpose() {
            let entry = match entry {
                Ok(entry) => entry,
                Err(err) => {
                    current_span.record("entries.read", &files_read);
                    current_span.record("entries.skipped", &files_skipped);
                    error!(?err, "Error reading directory");
                    set.shutdown().await;
                    return Err(LocalStorageError::from(err));
                }
            };
            let entry = entry;
            let path = entry.path();
            if path.is_file() && is_hidden_file(&path) {
                trace!(?path, "Skipping Meta File");
                files_skipped += 1;
                // Check if file is a meta file
                continue;
            }
            files_read += 1;
            let span_clone = current_span.clone();
            set.spawn_blocking(move || {
                span_clone.in_scope(|| StorageFileMeta::read_from_path(&path))
            });
        }
        current_span.record("entries.read", &files_read);
        current_span.record("entries.skipped", &files_skipped);
        let meta = StorageFileMeta::read_from_directory(path)?;

        let mut files = vec![];
        while let Some(res) = set.join_next().await {
            match res {
                Ok(Ok(ok)) => files.push(ok),
                Ok(Err(err)) => {
                    set.shutdown().await;
                    return Err(err);
                }
                Err(err) => {
                    error!(?err, "Some unknown error occurred in reading the file!");
                    set.shutdown().await;
                    return Err(LocalStorageError::other(err));
                }
            }
        }
        Ok(StorageFile::Directory { meta, files })
    }

    pub async fn update_meta_and_parent_metas(
        &self,
        path: &PathBuf,
        greatest_parent: Option<PathBuf>,
    ) -> Result<usize, LocalStorageError> {
        let mut metas_updated = 0;
        if let Some(greatest_parent) = greatest_parent {
            if let Some(parent) = path.parent() {
                if parent == self.config.path {
                    trace!("Do not update root directory");
                } else {
                    metas_updated += 1;
                    self.meta_update_sender
                        .send(parent.to_path_buf())
                        .await
                        .unwrap();
                }
            }
            let mut next_path = greatest_parent.clone();
            for part in path.strip_prefix(&greatest_parent).unwrap().components() {
                event!(Level::DEBUG, ?next_path, "Updating Meta");
                self.meta_update_sender
                    .send(next_path.clone())
                    .await
                    .unwrap();
                metas_updated += 1;
                next_path = next_path.join(part);
            }
        } else {
            self.meta_update_sender.send(path.clone()).await.unwrap();
            metas_updated += 1;
            let parent = path.parent();
            if let Some(parent) = parent {
                metas_updated += 1;
                self.meta_update_sender
                    .send(parent.to_path_buf())
                    .await
                    .unwrap();
            }
        }

        Ok(metas_updated)
    }
}
impl LocalStorage {
    pub fn run_post_save_file(
        self,
        path: PathBuf,
        new_directory_start: Option<PathBuf>,
        span: Span,
    ) -> Result<(), LocalStorageError> {
        let post_save_span = debug_span!(
            parent: &span,
            "Post Save File",
            metas.updated = Empty,
            file.path = debug(&path),
            new.dir = ?new_directory_start,
            otel.exception = Empty,
            otel.status_code = Empty,
        );
        tokio::task::spawn(async move {
            let _guard = post_save_span.enter();
            match self
                .0
                .update_meta_and_parent_metas(&path, new_directory_start)
                .await
            {
                Ok(ok) => {
                    post_save_span.record("metas.updated", &ok);
                    post_save_span.record("otel.status_code", "OK");
                    debug!(metas.updated = ok, "Updated Metas");
                }
                Err(err) => {
                    span.record("exception.message", &err.to_string());
                    span.record("otel.status_code", "ERROR");
                    event!(Level::ERROR, ?err, "Error Updating Metas");
                }
            }
        });
        Ok(())
    }
}
impl Storage for LocalStorage {
    type Error = LocalStorageError;
    fn storage_type_name(&self) -> &'static str {
        "Local"
    }
    fn storage_config(&self) -> BorrowedStorageConfig<'_> {
        BorrowedStorageConfig {
            storage_config: &self.storage_config,
            config: BorrowedStorageTypeConfig::Local(&self.config),
        }
    }
    #[instrument(
        fields(
            storage.type = "local",
            content.length = ?content.content_len_or_none(),
            storage.id = %self.storage_config.storage_id,
            storage.config = ?self.config,
            file.new,
            file.path,
            repository.id = %repository,
        ),
        skip(self,content, repository)
    )]
    async fn save_file(
        &self,
        repository: Uuid,
        content: FileContent,
        location: &StoragePath,
    ) -> Result<(usize, bool), LocalStorageError> {
        let CreatePath {
            path,
            parent_directory,
            new_directory_start,
        } = self.0.get_path_for_creation(repository, location)?;
        if new_directory_start.is_some() {
            trace!("Creating Parent Directory");
            fs::create_dir_all(parent_directory)?;
        }
        let current_span = Span::current();
        let new_file = !path.exists();
        current_span.record("file.new", &new_file);
        current_span.record("file.path", debug(&path));
        debug!(?path, "Saving File");
        let mut file = fs::File::create(&path)?;
        let bytes_written = content.write_to(&mut file)?;
        if !is_hidden_file(&path) {
            // Don't run post save file for meta files
            self.clone()
                .run_post_save_file(path, new_directory_start, current_span)?;
        }
        Ok((bytes_written, new_file))
    }
    #[instrument(
        fields(
            storage.type = "local",
            storage.id = %self.storage_config.storage_id,
            storage.config = ?self.config,
            repository.id = %repository,
        ),
        skip(self,repository)
    )]
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
            LocationMeta::delete_local(path)?;
        }
        Ok(true)
    }
    #[instrument(
        fields(
            storage.type = "local",
            storage.id = %self.storage_config.storage_id,
            storage.config = ?self.config,
        ),
        skip(self)
    )]
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
    #[instrument(
        fields(
            storage.type = "local",
            storage.id = %self.storage_config.storage_id,
            storage.config = ?self.config,
        ),
        skip(self)
    )]
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
    #[instrument(
        fields(
            storage.type = "local",
            storage.id = %self.storage_config.storage_id,
            storage.config = ?self.config,
        ),
        skip(self)
    )]
    async fn unload(&self) -> Result<(), LocalStorageError> {
        info!(?self, "Unloading Local Storage");
        let shutdown_signal = self.0.shutdown_signal.lock().await.take();
        if let Some(shutdown_signal) = shutdown_signal {
            shutdown_signal.send(()).unwrap();
        } else {
            error!("Shutdown Signal already sent");
        }
        // TODO: Implement Unload
        Ok(())
    }
    #[instrument(
        fields(
            storage.type = "local",
            storage.id = %self.storage_config.storage_id,
            storage.config = ?self.config,
        ),
        skip(self)
    )]
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
    #[instrument(
        fields(
            storage.type = "local",
            storage.id = %self.storage_config.storage_id,
            storage.config = ?self.config,
        ),
        skip(self)
    )]
    async fn get_repository_meta(
        &self,
        repository: Uuid,
        location: &StoragePath,
    ) -> Result<Option<RepositoryMeta>, LocalStorageError> {
        let path = self.get_path(&repository, location);
        if !path.exists() {
            return Ok(None);
        }
        let meta = LocationMeta::get_or_default_local(&path).map(|(meta, _)| meta)?;
        Ok(Some(meta.repository_meta))
    }
    #[instrument(
        fields(
            storage.type = "local",
            storage.id = %self.storage_config.storage_id,
            storage.config = ?self.config,
        ),
        skip(self)
    )]
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

        LocationMeta::set_repository_meta(path, value)?;
        Ok(())
    }
    #[instrument(
        fields(
            storage.type = "local",
            storage.id = %self.storage_config.storage_id,
            storage.config = ?self.config,
        ),
        skip(self)
    )]
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
        let (shutdown_signal, shutdown_receiver) = tokio::sync::oneshot::channel();
        let (meta_update_sender, meta_update_receiver) = tokio::sync::mpsc::channel(100);
        let inner = LocalStorageInner {
            config: type_config,
            storage_config: inner,
            shutdown_signal: Mutex::new(Some(shutdown_signal)),
            meta_update_sender,
        };
        meta_update_task(shutdown_receiver, meta_update_receiver);
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

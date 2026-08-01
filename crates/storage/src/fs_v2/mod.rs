//! FileSystem V2 — on-disk storage where each artifact is a single self-describing object.
//!
//! # How it differs from [crate::local]
//!
//! The original local backend stores an artifact as the raw bytes plus a sibling `.nr-meta` file
//! holding postcard-encoded metadata (and a `.nr-meta` inside every directory). That has three
//! costs this backend does not pay:
//!
//! - **Two files per artifact.** Every listing has to filter hidden entries, and the two can drift
//!   apart — a sidecar that fails to decode is silently rebuilt.
//! - **Eventually consistent metadata.** Sidecars are written by a background task fed over a
//!   channel, so a file's hashes are briefly wrong after it is written.
//! - **Whole-file reads to hash.** `generate_hashes_from_path` reads the file into a `Vec` after
//!   the fact.
//!
//! Here an object is one file in the [tux_io_encoding] layout: a 32 byte header, a metadata map, a
//! tag map, then the content. Writing streams the content first and the prefix last, so hashes are
//! computed from the bytes as they go past and written in the same pass. The write lands in a temp
//! file that is renamed into place, so a reader never sees a half-written artifact.
//!
//! # Where things are stored
//!
//! - **Metadata map** (system controlled): `created`, `modified`, `content-type`, and one entry per
//!   digest. Keys are [tux_io_encoding::MetaKey], which wraps `http::HeaderName` — so they are
//!   lowercase and header-shaped.
//! - **Tags** (user controlled): a single `nr-repository-meta` entry holding a postcard-encoded
//!   [RepositoryMeta]. One opaque blob rather than a tag per field, because `extra_meta` keys are
//!   arbitrary strings and would not all survive being mapped onto tag keys.
//! - **Directories** are real directories. Their metadata lives in a `.nr-dir` object inside them,
//!   which is what gives a directory somewhere to carry [RepositoryMeta] — Maven stamps project
//!   ids onto directories, not just files.
//!
//! Range reads come for free here (`content_range_reader`), which the original local backend
//! cannot do.
use std::{
    io::{ErrorKind, Write},
    ops::Deref,
    path::{Path, PathBuf},
    sync::Arc,
};

use chrono::{DateTime, FixedOffset, Local};
use futures::future::BoxFuture;
use nr_core::storage::{FileHashes, SerdeMime, StoragePath};
use serde::{Deserialize, Serialize};
use tokio::io::AsyncWriteExt;
use tracing::{debug, info, instrument, warn};
use tux_io_encoding::{
    CompressionTypes, MetaKey, Tags, ValueType,
    compression_types::{GzipCompressionType, ZStdCompressionType},
    fs::{AsyncTuxObject, CreateOptions, TuxObject},
};
use utoipa::ToSchema;
use uuid::Uuid;

pub mod error;
use error::FsV2StorageError;

use crate::{
    BorrowedStorageConfig, BorrowedStorageTypeConfig, DirectoryFileType, DynStorage, FileContent,
    FileFileType, FileType, PathCollisionError, StaticStorageFactory, Storage, StorageConfig,
    StorageConfigInner, StorageError, StorageFactory, StorageFile, StorageFileMeta,
    StorageFileReader, StorageTypeConfig, StorageTypeConfigTrait, meta::RepositoryMeta,
    streaming::VecDirectoryListStream, utils::new_type_arc_type,
};

/// Object inside a directory carrying that directory's metadata.
const DIRECTORY_META_FILE: &str = ".nr-dir";
/// Tag holding the postcard-encoded [RepositoryMeta].
const REPOSITORY_META_TAG: &str = "nr-repository-meta";

mod meta_keys {
    use http::HeaderName;
    pub const CREATED: HeaderName = HeaderName::from_static("created");
    pub const MODIFIED: HeaderName = HeaderName::from_static("modified");
    pub const CONTENT_TYPE: HeaderName = HeaderName::from_static("content-type");
    pub const MD5: HeaderName = HeaderName::from_static("md5");
    pub const SHA1: HeaderName = HeaderName::from_static("sha1");
    pub const SHA2_256: HeaderName = HeaderName::from_static("sha2-256");
    pub const SHA3_256: HeaderName = HeaderName::from_static("sha3-256");
}

/// Content compression applied to stored artifacts.
///
/// Off by default: most artifacts (jars, tarballs, `.tgz`) are already compressed, so the codec
/// costs CPU for nothing. It is worth turning on for a repository of POMs and metadata XML.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum Compression {
    #[default]
    None,
    Zstd,
    Gzip,
}
impl From<Compression> for CompressionTypes {
    fn from(value: Compression) -> Self {
        match value {
            Compression::None => CompressionTypes::default(),
            // Mid-range levels: the point here is to shrink text-ish artifacts (POMs, metadata
            // XML, packuments) without making a deploy wait on the codec.
            Compression::Zstd => CompressionTypes::ZSTD(ZStdCompressionType(3)),
            Compression::Gzip => CompressionTypes::Gzip(GzipCompressionType(6)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct FileSystemV2Config {
    /// Root directory holding one sub-directory per repository.
    #[schema(value_type = String)]
    pub path: PathBuf,
    #[serde(default)]
    pub compression: Compression,
    /// `fsync` each object before publishing it.
    ///
    /// Costs a flush per write in exchange for a completed upload surviving a power loss. Off by
    /// default because the rename is already atomic — a crash loses the write rather than
    /// corrupting anything.
    #[serde(default)]
    pub sync: bool,
}

#[derive(Debug)]
pub struct FileSystemV2Inner {
    pub config: FileSystemV2Config,
    pub storage_config: StorageConfigInner,
}

impl FileSystemV2Inner {
    /// Absolute path for a repository-relative location.
    fn resolve(&self, repository: Uuid, location: &StoragePath) -> PathBuf {
        let mut path = self.config.path.join(repository.to_string());
        for component in location.clone() {
            path.push(component.as_ref());
        }
        path
    }

    /// Rejects a path whose ancestor is an existing artifact.
    ///
    /// `/a/b` being a file makes `/a/b/c` impossible; without this check the write would fail
    /// later with a bare `NotADirectory` from the OS.
    fn check_for_collisions(
        &self,
        repository: Uuid,
        location: &StoragePath,
    ) -> Result<(), FsV2StorageError> {
        let mut path = self.config.path.join(repository.to_string());
        let mut conflicting_path = StoragePath::default();
        let mut components = location.clone().into_iter().peekable();
        while let Some(component) = components.next() {
            path.push(component.as_ref());
            conflicting_path.push_mut(component.as_ref());
            if components.peek().is_none() {
                break;
            }
            if path.is_file() {
                return Err(PathCollisionError {
                    path: location.clone(),
                    conflicts_with: conflicting_path,
                }
                .into());
            }
        }
        Ok(())
    }

    /// Where a directory keeps its own metadata.
    fn directory_meta_path(directory: &Path) -> PathBuf {
        directory.join(DIRECTORY_META_FILE)
    }

    /// Whether a directory entry is bookkeeping rather than a stored artifact.
    fn is_hidden(name: &str) -> bool {
        name == DIRECTORY_META_FILE
    }

    /// Opens the object backing a path, whether it is a file or a directory.
    async fn open_object(
        &self,
        repository: Uuid,
        location: &StoragePath,
    ) -> Result<Option<AsyncTuxObject>, FsV2StorageError> {
        let path = self.resolve(repository, location);
        let path = if path.is_dir() {
            Self::directory_meta_path(&path)
        } else {
            path
        };
        Ok(AsyncTuxObject::open_optional(path).await?)
    }

    /// Writes a directory's metadata object, creating the directory if needed.
    async fn write_directory_meta(
        &self,
        directory: &Path,
        repository_meta: RepositoryMeta,
    ) -> Result<(), FsV2StorageError> {
        tokio::fs::create_dir_all(directory).await?;
        let now = Local::now().fixed_offset();
        let meta_path = Self::directory_meta_path(directory);

        let created = match AsyncTuxObject::open_optional(&meta_path).await? {
            Some(existing) => {
                read_datetime(existing.metadata(), &meta_keys::CREATED).unwrap_or(now)
            }
            None => now,
        };

        let mut options = CreateOptions::new().with_sync(self.config.sync);
        options
            .metadata
            .insert(MetaKey::from(meta_keys::CREATED), ValueType::from(created));
        options
            .metadata
            .insert(MetaKey::from(meta_keys::MODIFIED), ValueType::from(now));
        options.tags = repository_meta_tags(&repository_meta)?;

        let writer = AsyncTuxObject::create(&meta_path, options).await?;
        writer.finish().await?;
        Ok(())
    }

    /// Reads a directory's [RepositoryMeta], if it has any.
    async fn read_directory_meta(
        &self,
        directory: &Path,
    ) -> Result<Option<AsyncTuxObject>, FsV2StorageError> {
        Ok(AsyncTuxObject::open_optional(Self::directory_meta_path(directory)).await?)
    }

    /// Builds the listing entry for one filesystem path.
    async fn meta_for_path(
        &self,
        path: &Path,
        name: String,
    ) -> Result<Option<StorageFileMeta<FileType>>, FsV2StorageError> {
        if path.is_dir() {
            let entries = self.count_entries(path).await?;
            let (created, modified) = match self.read_directory_meta(path).await? {
                Some(object) => timestamps_from(object.metadata()),
                None => (None, None),
            };
            let modified = modified.unwrap_or_else(|| Local::now().fixed_offset());
            return Ok(Some(StorageFileMeta {
                name,
                file_type: FileType::Directory(DirectoryFileType {
                    file_count: entries,
                }),
                modified,
                created: created.unwrap_or(modified),
            }));
        }

        let Some(object) = AsyncTuxObject::open_optional(path).await? else {
            return Ok(None);
        };
        let metadata = object.metadata();
        let (created, modified) = timestamps_from(metadata);
        let modified = modified.unwrap_or_else(|| Local::now().fixed_offset());
        Ok(Some(StorageFileMeta {
            name,
            file_type: FileType::File(FileFileType {
                file_size: object.content_length(),
                mime_type: metadata
                    .get_header(&meta_keys::CONTENT_TYPE)
                    .and_then(ValueType::as_str)
                    .and_then(|value| value.parse().ok())
                    .map(SerdeMime),
                file_hash: hashes_from(metadata),
            }),
            modified,
            created: created.unwrap_or(modified),
        }))
    }

    /// Counts the visible entries in a directory.
    async fn count_entries(&self, directory: &Path) -> Result<u64, FsV2StorageError> {
        let mut entries = tokio::fs::read_dir(directory).await?;
        let mut count = 0;
        while let Some(entry) = entries.next_entry().await? {
            if !Self::is_hidden(&entry.file_name().to_string_lossy()) {
                count += 1;
            }
        }
        Ok(count)
    }

    /// Lists one directory level.
    async fn list_directory(
        &self,
        directory: &Path,
    ) -> Result<Vec<StorageFileMeta<FileType>>, FsV2StorageError> {
        let mut files = Vec::new();
        let mut entries = tokio::fs::read_dir(directory).await?;
        while let Some(entry) = entries.next_entry().await? {
            let name = entry.file_name().to_string_lossy().into_owned();
            if Self::is_hidden(&name) {
                continue;
            }
            if let Some(meta) = self.meta_for_path(&entry.path(), name).await? {
                files.push(meta);
            }
        }
        Ok(files)
    }
}

/// Encodes [RepositoryMeta] into the tag map an object is created with.
fn repository_meta_tags(meta: &RepositoryMeta) -> Result<Tags, FsV2StorageError> {
    let mut tags = Tags::new();
    tags.insert(
        REPOSITORY_META_TAG.to_owned(),
        ValueType::Bytes(postcard::to_allocvec(meta)?),
    );
    Ok(tags)
}

/// Decodes the [RepositoryMeta] tag, treating an unreadable one as absent.
fn repository_meta_from_tags(tags: &Tags) -> RepositoryMeta {
    let Some(ValueType::Bytes(bytes)) = tags.get(REPOSITORY_META_TAG) else {
        return RepositoryMeta::default();
    };
    match postcard::from_bytes(bytes) {
        Ok(meta) => meta,
        Err(err) => {
            warn!(?err, "Repository meta tag is unreadable; treating as empty");
            RepositoryMeta::default()
        }
    }
}

fn read_datetime(
    metadata: &tux_io_encoding::MetadataMap,
    key: &http::HeaderName,
) -> Option<DateTime<FixedOffset>> {
    match metadata.get_header(key) {
        Some(ValueType::RawDateTime(raw)) => DateTime::try_from(*raw).ok(),
        _ => None,
    }
}

fn timestamps_from(
    metadata: &tux_io_encoding::MetadataMap,
) -> (Option<DateTime<FixedOffset>>, Option<DateTime<FixedOffset>>) {
    (
        read_datetime(metadata, &meta_keys::CREATED),
        read_datetime(metadata, &meta_keys::MODIFIED),
    )
}

fn hashes_from(metadata: &tux_io_encoding::MetadataMap) -> FileHashes {
    let read = |name: &http::HeaderName| {
        metadata
            .get_header(name)
            .and_then(ValueType::as_str)
            .map(str::to_owned)
    };
    FileHashes {
        md5: read(&meta_keys::MD5),
        sha1: read(&meta_keys::SHA1),
        sha2_256: read(&meta_keys::SHA2_256),
        sha3_256: read(&meta_keys::SHA3_256),
    }
}

fn insert_hashes(metadata: &mut tux_io_encoding::MetadataMap, hashes: &FileHashes) {
    let mut insert = |name: http::HeaderName, value: &Option<String>| {
        if let Some(value) = value {
            metadata.insert(MetaKey::from(name), ValueType::String(value.clone()));
        }
    };
    insert(meta_keys::MD5, &hashes.md5);
    insert(meta_keys::SHA1, &hashes.sha1);
    insert(meta_keys::SHA2_256, &hashes.sha2_256);
    insert(meta_keys::SHA3_256, &hashes.sha3_256);
}

#[derive(Debug, Clone)]
pub struct FileSystemV2Storage(Arc<FileSystemV2Inner>);
new_type_arc_type!(FileSystemV2Storage(FileSystemV2Inner));

impl Storage for FileSystemV2Storage {
    type Error = FsV2StorageError;
    type DirectoryStream = VecDirectoryListStream;

    fn storage_type_name(&self) -> &'static str {
        FileSystemV2Factory::STORAGE_TYPE_NAME
    }

    #[instrument(
        name = "Storage::unload",
        skip(self),
        fields(storage_type = "FileSystemV2")
    )]
    async fn unload(&self) -> Result<(), FsV2StorageError> {
        // Every write is published by rename before `save_file` returns, so there is nothing in
        // flight to wait for — unlike the original local backend's background meta task.
        info!("Unloading FileSystemV2 Storage");
        Ok(())
    }

    fn storage_config(&self) -> BorrowedStorageConfig<'_> {
        BorrowedStorageConfig {
            storage_config: &self.storage_config,
            config: BorrowedStorageTypeConfig::FileSystemV2(&self.config),
        }
    }

    #[instrument(
        name = "Storage::save_file",
        skip(self, file),
        fields(storage_type = "FileSystemV2")
    )]
    async fn save_file(
        &self,
        repository: Uuid,
        file: FileContent,
        location: &StoragePath,
    ) -> Result<(usize, bool), FsV2StorageError> {
        self.check_for_collisions(repository, location)?;

        let path = self.resolve(repository, location);
        let parent = path
            .parent()
            .ok_or_else(|| FsV2StorageError::NoParentDirectory(location.clone()))?;
        tokio::fs::create_dir_all(parent).await?;

        let now = Local::now().fixed_offset();
        // Carry forward whatever the previous version of this artifact recorded, so an overwrite
        // does not reset the creation time or drop the project ids Maven attached to the path.
        let (created, existing_repository_meta, already_exists) =
            match AsyncTuxObject::open_optional(&path).await? {
                Some(mut existing) => {
                    let created =
                        read_datetime(existing.metadata(), &meta_keys::CREATED).unwrap_or(now);
                    let tags = existing.read_tags().await?;
                    (created, repository_meta_from_tags(&tags), true)
                }
                None => (now, RepositoryMeta::default(), false),
            };

        let hashes = file.generate_hashes()?;
        let bytes: Vec<u8> = file.try_into()?;
        let size = bytes.len();

        let mut options = CreateOptions::new()
            .with_sync(self.config.sync)
            .with_compression(self.config.compression.into());
        options
            .metadata
            .insert(MetaKey::from(meta_keys::CREATED), ValueType::from(created));
        options
            .metadata
            .insert(MetaKey::from(meta_keys::MODIFIED), ValueType::from(now));
        if let Some(mime) = mime_guess::from_path(&path).first() {
            options.metadata.insert(
                MetaKey::from(meta_keys::CONTENT_TYPE),
                ValueType::String(mime.to_string()),
            );
        }
        insert_hashes(&mut options.metadata, &hashes);
        options.tags = repository_meta_tags(&existing_repository_meta)?;

        if matches!(self.config.compression, Compression::None) {
            let mut writer = AsyncTuxObject::create(&path, options).await?;
            writer.write_all(&bytes).await?;
            writer.finish().await?;
        } else {
            // The compression codecs are synchronous, and `AsyncObjectWriter::create` rejects a
            // compressed object outright for that reason. Compressed writes therefore go through
            // the blocking writer on the blocking pool rather than the async one.
            let path = path.clone();
            tokio::task::spawn_blocking(move || -> Result<(), FsV2StorageError> {
                let mut writer = TuxObject::create(&path, options)?;
                let mut encoder = writer.content_encoder()?;
                encoder.write_all(&bytes)?;
                encoder.finish()?;
                writer.finish()?;
                Ok(())
            })
            .await
            .map_err(FsV2StorageError::from)??;
        }

        // Keep the parent directory's own object present so it can carry metadata and timestamps.
        if self.read_directory_meta(parent).await?.is_none() {
            self.write_directory_meta(parent, RepositoryMeta::default())
                .await?;
        }

        Ok((size, !already_exists))
    }

    #[instrument(
        name = "Storage::put_repository_meta",
        skip(self),
        fields(storage_type = "FileSystemV2")
    )]
    async fn put_repository_meta(
        &self,
        repository: Uuid,
        location: &StoragePath,
        value: RepositoryMeta,
    ) -> Result<(), FsV2StorageError> {
        let path = self.resolve(repository, location);
        if path.is_dir() {
            return self.write_directory_meta(&path, value).await;
        }

        let Some(mut object) = AsyncTuxObject::open_writable(&path).await.map_or_else(
            |err| match err {
                tux_io_encoding::fs::ObjectFileError::IO(io)
                    if io.kind() == ErrorKind::NotFound =>
                {
                    Ok(None)
                }
                other => Err(other),
            },
            |object| Ok(Some(object)),
        )?
        else {
            return Err(FsV2StorageError::NotFound(location.clone()));
        };
        // Only the tag section changes, and it is rewritten atomically.
        object.set_tags(repository_meta_tags(&value)?).await?;
        Ok(())
    }

    #[instrument(
        name = "Storage::get_repository_meta",
        skip(self),
        fields(storage_type = "FileSystemV2")
    )]
    async fn get_repository_meta(
        &self,
        repository: Uuid,
        location: &StoragePath,
    ) -> Result<Option<RepositoryMeta>, FsV2StorageError> {
        let Some(mut object) = self.open_object(repository, location).await? else {
            return Ok(None);
        };
        let tags = object.read_tags().await?;
        Ok(Some(repository_meta_from_tags(&tags)))
    }

    #[instrument(
        name = "Storage::delete_file",
        skip(self),
        fields(storage_type = "FileSystemV2")
    )]
    async fn delete_file(
        &self,
        repository: Uuid,
        location: &StoragePath,
    ) -> Result<bool, FsV2StorageError> {
        let path = self.resolve(repository, location);
        if path.is_dir() {
            tokio::fs::remove_dir_all(&path).await?;
            return Ok(true);
        }
        match tokio::fs::remove_file(&path).await {
            Ok(()) => Ok(true),
            Err(err) if err.kind() == ErrorKind::NotFound => Ok(false),
            Err(err) => Err(err.into()),
        }
    }

    #[instrument(
        name = "Storage::get_file_information",
        skip(self),
        fields(storage_type = "FileSystemV2")
    )]
    async fn get_file_information(
        &self,
        repository: Uuid,
        location: &StoragePath,
    ) -> Result<Option<StorageFileMeta<FileType>>, FsV2StorageError> {
        let path = self.resolve(repository, location);
        if !path.exists() {
            return Ok(None);
        }
        let name = location
            .clone()
            .into_iter()
            .next_back()
            .map(|component| component.to_string())
            .unwrap_or_default();
        self.meta_for_path(&path, name).await
    }

    #[instrument(
        name = "Storage::open_file",
        skip(self),
        fields(storage_type = "FileSystemV2")
    )]
    async fn open_file(
        &self,
        repository: Uuid,
        location: &StoragePath,
    ) -> Result<Option<StorageFile>, FsV2StorageError> {
        let path = self.resolve(repository, location);
        let Some(meta) = self.get_file_information(repository, location).await? else {
            return Ok(None);
        };

        match meta.file_type {
            FileType::Directory(file_type) => Ok(Some(StorageFile::Directory {
                meta: StorageFileMeta {
                    name: meta.name,
                    file_type,
                    modified: meta.modified,
                    created: meta.created,
                },
                files: self.list_directory(&path).await?,
            })),
            FileType::File(file_type) => {
                let Some(object) = AsyncTuxObject::open_optional(&path).await? else {
                    return Ok(None);
                };
                // `into_content_reader` hands over the file handle, so the reader can outlive this
                // function and be handed straight to a response body.
                let reader = if object.is_compressed() {
                    // Compressed content has to be decoded, and the codecs are synchronous.
                    let mut object = object;
                    let content = object.read_content_to_vec().await?;
                    StorageFileReader::from(crate::FileContentBytes::Content(content))
                } else {
                    StorageFileReader::AsyncReader(Box::pin(object.into_content_reader().await?))
                };
                Ok(Some(StorageFile::File {
                    meta: StorageFileMeta {
                        name: meta.name,
                        file_type,
                        modified: meta.modified,
                        created: meta.created,
                    },
                    content: reader,
                }))
            }
        }
    }

    #[instrument(
        name = "Storage::validate_config_change",
        skip(self),
        fields(storage_type = "FileSystemV2")
    )]
    async fn validate_config_change(
        &self,
        config: StorageTypeConfig,
    ) -> Result<(), FsV2StorageError> {
        let config = FileSystemV2Config::from_type_config(config)?;
        if config.path != self.config.path {
            // Moving the root would orphan everything already stored under the old one.
            return Err(FsV2StorageError::PathCannotBeChanged);
        }
        Ok(())
    }

    #[instrument(
        name = "Storage::file_exists",
        skip(self),
        fields(storage_type = "FileSystemV2")
    )]
    async fn file_exists(
        &self,
        repository: Uuid,
        location: &StoragePath,
    ) -> Result<bool, FsV2StorageError> {
        Ok(self.resolve(repository, location).exists())
    }

    #[instrument(
        name = "Storage::stream_directory",
        skip(self),
        fields(storage_type = "FileSystemV2")
    )]
    async fn stream_directory(
        &self,
        repository: Uuid,
        location: &StoragePath,
    ) -> Result<Option<Self::DirectoryStream>, FsV2StorageError> {
        let path = self.resolve(repository, location);
        if !path.is_dir() {
            return Ok(None);
        }
        let files = self.list_directory(&path).await?;
        let name = location
            .clone()
            .into_iter()
            .next_back()
            .map(|component| component.to_string())
            .unwrap_or_default();
        let (created, modified) = match self.read_directory_meta(&path).await? {
            Some(object) => timestamps_from(object.metadata()),
            None => (None, None),
        };
        let modified = modified.unwrap_or_else(|| Local::now().fixed_offset());
        Ok(Some(VecDirectoryListStream::new(
            files.clone(),
            StorageFileMeta {
                name,
                file_type: DirectoryFileType {
                    file_count: files.len() as u64,
                },
                modified,
                created: created.unwrap_or(modified),
            },
        )))
    }
}

#[derive(Debug, Default)]
pub struct FileSystemV2Factory;

impl FileSystemV2Factory {
    pub const STORAGE_TYPE_NAME: &'static str = "FileSystemV2";

    async fn prepare(config: &FileSystemV2Config) -> Result<(), FsV2StorageError> {
        if config.path.is_file() {
            return Err(FsV2StorageError::RootIsAFile(config.path.clone()));
        }
        tokio::fs::create_dir_all(&config.path).await?;
        debug!(path = ?config.path, "FileSystemV2 root ready");
        Ok(())
    }
}

impl StaticStorageFactory for FileSystemV2Factory {
    type StorageType = FileSystemV2Storage;
    type ConfigType = FileSystemV2Config;
    type Error = FsV2StorageError;

    fn storage_type_name() -> &'static str {
        Self::STORAGE_TYPE_NAME
    }

    async fn test_storage_config(config: StorageTypeConfig) -> Result<(), FsV2StorageError> {
        Self::prepare(&FileSystemV2Config::from_type_config(config)?).await
    }

    async fn create_storage(
        inner: StorageConfigInner,
        type_config: Self::ConfigType,
    ) -> Result<Self::StorageType, FsV2StorageError> {
        Self::prepare(&type_config).await?;
        Ok(FileSystemV2Storage::from(FileSystemV2Inner {
            config: type_config,
            storage_config: inner,
        }))
    }
}

impl StorageFactory for FileSystemV2Factory {
    fn storage_name(&self) -> &'static str {
        Self::STORAGE_TYPE_NAME
    }

    fn test_storage_config(
        &self,
        config: StorageTypeConfig,
    ) -> BoxFuture<'static, Result<(), StorageError>> {
        Box::pin(async move {
            <Self as StaticStorageFactory>::test_storage_config(config).await?;
            Ok(())
        })
    }

    fn create_storage(
        &self,
        config: StorageConfig,
    ) -> BoxFuture<'static, Result<DynStorage, StorageError>> {
        Box::pin(async move {
            let storage =
                <Self as StaticStorageFactory>::create_storage_from_config(config).await?;
            Ok(DynStorage::FileSystemV2(storage))
        })
    }
}

#[cfg(test)]
mod tests {
    use nr_core::storage::StoragePath;
    use tracing::warn;
    use uuid::Uuid;

    use super::*;
    use crate::{
        StaticStorageFactory, fs_v2::FileSystemV2Factory, testing::storage::TestingStorage,
    };

    /// Compressed objects take a different write path to uncompressed ones.
    ///
    /// `AsyncObjectWriter::create` rejects compression outright — the codecs are synchronous — so
    /// compressed writes go through the blocking writer instead. Nothing else covers that branch,
    /// and a mistake in it would store content whose header claims a codec that was never applied,
    /// which reads back as garbage rather than failing loudly.
    async fn round_trip_with_compression(compression: Compression) -> anyhow::Result<()> {
        let root = std::env::temp_dir().join(format!("nr-fs-v2-{}", Uuid::new_v4()));
        let storage = <FileSystemV2Factory as StaticStorageFactory>::create_storage(
            StorageConfigInner::test_config(),
            FileSystemV2Config {
                path: root.clone(),
                compression,
                sync: false,
            },
        )
        .await?;

        let repository = Uuid::new_v4();
        let path = StoragePath::from("compressed/pom.xml");
        // Repetitive enough that a codec has something to do.
        let body = "<project><modelVersion>4.0.0</modelVersion></project>".repeat(64);

        let (written, is_new) = storage
            .save_file(repository, FileContent::from(body.as_str()), &path)
            .await?;
        assert_eq!(written, body.len());
        assert!(is_new);

        let Some(StorageFile::File { meta, content }) =
            storage.open_file(repository, &path).await?
        else {
            panic!("Compressed object did not read back as a file");
        };
        let read = content.read_to_vec(body.len()).await?;
        assert_eq!(
            String::from_utf8(read)?,
            body,
            "Content did not survive a {compression:?} round trip"
        );
        assert!(
            meta.file_type.file_hash.sha2_256.is_some(),
            "Hashes should describe the original content, not the compressed bytes"
        );

        tokio::fs::remove_dir_all(&root).await?;
        Ok(())
    }

    #[tokio::test]
    async fn zstd_round_trip() -> anyhow::Result<()> {
        round_trip_with_compression(Compression::Zstd).await
    }

    #[tokio::test]
    async fn gzip_round_trip() -> anyhow::Result<()> {
        round_trip_with_compression(Compression::Gzip).await
    }

    #[tokio::test]
    async fn uncompressed_round_trip() -> anyhow::Result<()> {
        round_trip_with_compression(Compression::None).await
    }

    #[tokio::test]
    pub async fn generic_test() -> anyhow::Result<()> {
        let Some(config) = crate::testing::start_storage_test("FileSystemV2").await? else {
            warn!("FileSystemV2 Storage Test Skipped");
            return Ok(());
        };
        let storage =
            <FileSystemV2Factory as StaticStorageFactory>::create_storage_from_config(config)
                .await?;
        let testing_storage = TestingStorage::new(storage);
        crate::testing::tests::full_test(testing_storage).await?;

        Ok(())
    }
}

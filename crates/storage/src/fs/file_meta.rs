use chrono::{DateTime, FixedOffset, Local};
use digest::Digest;
use mime::Mime;
use nr_core::utils::base64_utils;
use serde::{Deserialize, Serialize};
use std::{
    fs::File,
    io::{self, Read},
    path::{Path, PathBuf},
};
use tracing::{
    debug, event,
    field::{debug, display, Empty},
    instrument, trace, trace_span, warn, Level, Span,
};
use utoipa::ToSchema;

use crate::{
    fs::utils::MetadataUtils, local::error::LocalStorageError, meta::RepositoryMeta,
    path::PathUtils,
};
pub static HIDDEN_FILE_EXTENSIONS: &[&str] = &["nr-meta"];
pub static NITRO_REPO_META_EXTENSION: &str = "nr-meta";
pub static NITRO_REPO_META_FILE: &str = ".nr-meta";
#[instrument(level = "trace")]
pub fn is_hidden_file(path: &Path) -> bool {
    trace!(?path, "Checking if file is hidden");
    if let Some(extension) = path.extension().and_then(|v| v.to_str()) {
        trace!(?extension, "Checking if file extension is hidden");
        HIDDEN_FILE_EXTENSIONS.contains(&extension)
    } else if let Some(file_name) = path.file_name().and_then(|v| v.to_str()) {
        trace!(?file_name, "Checking if file is hidden");
        file_name.eq(NITRO_REPO_META_FILE)
    } else {
        false
    }
}
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Default, ToSchema)]
pub struct FileHashes {
    pub md5: Option<String>,
    pub sha1: Option<String>,
    pub sha2_256: Option<String>,
    pub sha3_256: Option<String>,
}
impl FileHashes {
    pub fn generate_from_path(path: impl AsRef<Path>) -> Result<FileHashes, io::Error> {
        let mut buffer = Vec::new();
        {
            let mut file = std::fs::File::open(path)?;
            file.read_to_end(&mut buffer)?;
        }
        Ok(Self::generate_from_bytes(&buffer))
    }
    #[instrument(skip(buffer))]
    pub fn generate_from_bytes(buffer: &[u8]) -> FileHashes {
        FileHashes {
            md5: Some(Self::generate_md5(buffer)),
            sha1: Some(Self::generate_sha1(buffer)),
            sha2_256: Some(Self::generate_sha2_256(buffer)),
            sha3_256: Some(Self::generate_sha3_256(buffer)),
        }
    }
    fn generate_md5(buffer: &[u8]) -> String {
        use md5::Md5;

        let mut hasher = Md5::new();
        hasher.update(buffer);
        let hash = hasher.finalize();
        base64_utils::encode(hash)
    }
    fn generate_sha1(buffer: &[u8]) -> String {
        use sha1::Sha1;

        let mut hasher = Sha1::new();
        hasher.update(buffer);
        let hash = hasher.finalize();
        base64_utils::encode(hash)
    }
    fn generate_sha2_256(buffer: &[u8]) -> String {
        use sha2::Sha256;

        let mut hasher = Sha256::new();
        hasher.update(buffer);
        let hash = hasher.finalize();
        base64_utils::encode(hash)
    }
    fn generate_sha3_256(buffer: &[u8]) -> String {
        use sha3::Sha3_256;

        let mut hasher = Sha3_256::new();
        hasher.update(buffer);
        let hash = hasher.finalize();
        base64_utils::encode(hash)
    }
}
pub const FILE_META_MIME: Mime = mime::APPLICATION_JSON;
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileMeta {
    /// None if the file is a directory or meta file
    pub hashes: Option<FileHashes>,
    pub created: DateTime<FixedOffset>,
    pub modified: DateTime<FixedOffset>,
    pub repository_meta: RepositoryMeta,
}

impl FileMeta {
    pub fn update_hashes(&mut self, path: impl AsRef<Path>) -> Result<(), LocalStorageError> {
        if path.as_ref().is_dir() {
            self.hashes = None;
        } else {
            self.hashes = Some(FileHashes::generate_from_path(path)?);
        }
        Ok(())
    }

    #[instrument(
        level = "debug",
        skip(path),
        fields(
            path = ?path.as_ref(),
        )
    )]
    pub(crate) fn create_meta_or_update(path: impl AsRef<Path>) -> Result<(), LocalStorageError> {
        let (mut meta, was_created) = Self::get_or_default_local(&path)?;
        if !was_created {
            event!(Level::DEBUG, path = ?path.as_ref(), "Updating Meta File");
            meta.update_hashes(&path)?;
            meta.modified = Local::now().into();
            meta.save_meta(path)?;
        }

        Ok(())
    }
    #[instrument(
        level = "debug",
        skip(path),
        fields(
            path = ?path.as_ref(),
            path.meta = Empty,
            created = Empty,
        )
    )]
    pub(crate) fn get_or_default_local(
        path: impl AsRef<Path>,
    ) -> Result<(FileMeta, bool), LocalStorageError> {
        let span = Span::current();
        let meta_path = meta_path(&path)?;
        span.record("path.meta", debug(&meta_path));
        if meta_path.exists() {
            trace!(?meta_path, "Meta File exists. Reading");
            match FileMeta::read_meta_file(&meta_path) {
                Ok(meta) => {
                    span.record("created", &false);
                    return Ok((meta, false));
                }
                Err(LocalStorageError::Postcard(err)) => {
                    event!(
                        Level::ERROR,
                        ?meta_path,
                        ?err,
                        "Meta File is corrupted. Rebuilding"
                    );
                }
                Err(err) => {
                    return Err(err);
                }
            }
        } else if tracing::enabled!(Level::DEBUG) {
            debug!(?meta_path, "Meta File does not exist. Generating");
        }
        span.record("created", &true);
        let (created, modified) = {
            let file = File::open(&path)?;
            let metadata = file.metadata()?;
            let modified = metadata.modified_as_chrono_or_now()?;
            let created = metadata.created_as_chrono_or_now()?;
            (created, modified)
        };
        let mut meta = FileMeta {
            hashes: None,
            created,
            modified,
            repository_meta: RepositoryMeta::default(),
        };
        meta.update_hashes(&path)?;
        meta.save_meta(&path)?;

        return Ok((meta, true));
    }

    #[instrument(
        level = "debug",
        skip(path),
        fields(
            path = ?path.as_ref(),
        )
    )]
    /// Assumes the path is the path to the `.nr-meta` file
    fn read_meta_file(path: impl AsRef<Path>) -> Result<FileMeta, LocalStorageError> {
        let mut file = File::open(path)?;

        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes)?;
        let meta: FileMeta = postcard::from_bytes(&bytes)?;
        Ok(meta)
    }

    #[instrument(
        level = "debug",
        skip(path),
        fields(
            path = ?path.as_ref(),
            path.meta = Empty,
        )
    )]
    pub(crate) fn delete_local(path: impl AsRef<Path>) -> Result<(), LocalStorageError> {
        let meta_path = meta_path(&path)?;
        Span::current().record("path.meta", debug(&meta_path));
        if !meta_path.exists() {
            warn!(?meta_path, "Meta File does not exist");
            return Ok(());
        }
        debug!(?meta_path, "Deleting Meta File");
        std::fs::remove_file(meta_path)?;
        Ok(())
    }
    #[instrument(
        level = "debug",
        skip(path),
        fields(
            path = ?path.as_ref(),
            path.meta = Empty,
            created = Empty,
        )
    )]
    pub(crate) fn save_meta(&self, path: impl AsRef<Path>) -> Result<(), LocalStorageError> {
        let span = Span::current();
        let meta_path = meta_path(path)?;
        span.record("path.meta", debug(&meta_path));
        span.record("created", !meta_path.exists());
        let file = File::create(meta_path)?;
        postcard::to_io(self, file)?;
        event!(Level::DEBUG, "Saved Meta File");
        Ok(())
    }

    #[instrument(
        level = "debug",
        skip(path),
        fields(
            path = ?path.as_ref(),
        )
    )]
    pub(crate) fn set_repository_meta(
        path: impl AsRef<Path>,
        repository_meta: RepositoryMeta,
    ) -> Result<(), LocalStorageError> {
        let (mut meta, _) = Self::get_or_default_local(&path)?;
        meta.repository_meta = repository_meta;
        meta.save_meta(path)
    }
}

fn meta_path(path: impl AsRef<Path>) -> Result<PathBuf, LocalStorageError> {
    let meta_path = path.as_ref().to_path_buf();
    let meta_path = if meta_path.is_dir() {
        meta_path.join(NITRO_REPO_META_FILE)
    } else {
        meta_path.add_extension(NITRO_REPO_META_EXTENSION)?
    };
    Ok(meta_path)
}

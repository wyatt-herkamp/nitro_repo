use chrono::{DateTime, FixedOffset, Local};
use digest::Digest;
use mime::Mime;
use nr_core::utils::base64_utils;
use serde::{Deserialize, Serialize};
use std::{
    fs::File,
    io::{self, Read},
    path::Path,
};
use tracing::{debug, instrument, warn};
use utoipa::ToSchema;
pub static HIDDEN_FILE_EXTENSIONS: &[&str] = &["nr-meta", "nr-repository-meta"];
pub static NITRO_REPO_META_EXTENSION: &str = "nr-meta";
pub static NITRO_REPO_REPOSITORY_META_EXTENSION: &str = "nr-repository-meta";

pub fn is_hidden_file(path: &Path) -> bool {
    if let Some(extension) = path.extension().and_then(|v| v.to_str()) {
        HIDDEN_FILE_EXTENSIONS.contains(&extension)
    } else {
        false
    }
}
use crate::utils::{MetadataUtils, PathUtils};
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

/// FIle Meta Data. This is stored a separate file. with an additional extension of `.nr-meta`
///
/// Example: `too-many-shortcuts.jar` -> `too-many-shortcuts.jar.nr-meta`
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileMeta {
    #[serde(default)]
    pub hashes: FileHashes,
    pub created: DateTime<FixedOffset>,
    pub modified: DateTime<FixedOffset>,
}
impl FileMeta {
    #[instrument(name = "FileMeta::get_or_create_local", skip(path))]
    pub fn get_or_create_local(path: impl AsRef<Path>) -> Result<FileMeta, io::Error> {
        let path = path.as_ref();
        let meta_path = path
            .to_path_buf()
            .add_extension(NITRO_REPO_META_EXTENSION)?;
        debug!(?meta_path, ?path, "Creating or Loading Meta File");
        if meta_path.exists() {
            return FileMeta::read_meta_file(&meta_path);
        } else {
            debug!(?meta_path, "Meta File does not exist. Generating");
            let hashes = FileHashes::generate_from_path(path)?;
            let (created, modified) = {
                let file = File::open(path)?;
                let metadata = file.metadata()?;
                let modified = metadata.modified_as_chrono_or_now()?;
                let created = metadata.created_as_chrono_or_now()?;
                (created, modified)
            };

            let meta = FileMeta {
                hashes,
                created,
                modified,
            };
            debug!(?meta, ?meta_path, "Writing Meta File");
            let file = File::create(meta_path)?;
            postcard::to_io(&meta, file).map_err(|error| {
                warn!(?error, "Failed to write Meta File");
                io::Error::new(io::ErrorKind::Other, error)
            })?;

            Ok(meta)
        }
    }
    #[instrument]
    pub(crate) fn create_meta_or_update(path: &Path) -> Result<(), io::Error> {
        let meta_path = path
            .to_path_buf()
            .add_extension(NITRO_REPO_META_EXTENSION)?;
        debug!(?meta_path);
        let hashes = FileHashes::generate_from_path(path)?;
        let (created, modified) = {
            if meta_path.exists() {
                let meta = FileMeta::read_meta_file(&meta_path)?;
                debug!(
                    ?meta,
                    ?meta_path,
                    "Loaded Old Meta File. This will be updated"
                );
                (meta.created, Local::now().fixed_offset())
            } else {
                (Local::now().fixed_offset(), Local::now().fixed_offset())
            }
        };

        let meta = FileMeta {
            hashes,
            created,
            modified,
        };
        debug!(?meta, ?meta_path, "Writing Meta File");
        let file = File::create(meta_path)?;
        postcard::to_io(&meta, file).map_err(|error| {
            warn!(?error, "Failed to write Meta File");
            io::Error::new(io::ErrorKind::Other, error)
        })?;
        Ok(())
    }
    /// Assumes the path is the path to the `.nr-meta` file
    fn read_meta_file(path: &Path) -> Result<FileMeta, io::Error> {
        let mut file = File::open(path)?;
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes)?;
        let meta: FileMeta = postcard::from_bytes(&bytes).map_err(|error| {
            warn!(?error, "Failed to read Meta File");
            io::Error::new(io::ErrorKind::Other, error)
        })?;
        Ok(meta)
    }
    #[instrument(skip(path))]
    pub(crate) fn delete_local(path: impl AsRef<Path>) -> Result<(), io::Error> {
        let meta_path = path.as_ref().with_extension("nr-meta");
        debug!(?meta_path, "Deleting Meta File");
        if !meta_path.exists() {
            warn!(?meta_path, "Meta File does not exist");
        }
        std::fs::remove_file(meta_path)
    }
}

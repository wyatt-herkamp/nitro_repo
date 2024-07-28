use std::{
    fs::File,
    io::{self, Read},
    path::Path,
};

use ahash::{HashMap, HashMapExt};
use chrono::{DateTime, FixedOffset};
use mime::Mime;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::{debug, instrument, warn};

use crate::utils::MetadataUtils;
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Default)]
pub struct FileHashes {
    pub sha256: Option<String>,
}
impl FileHashes {
    pub fn generate_from_path(path: impl AsRef<Path>) -> Result<FileHashes, io::Error> {
        let mut buffer = Vec::new();
        {
            let mut file = std::fs::File::open(path)?;
            file.read_to_end(&mut buffer)?;
        }
        Ok(FileHashes {
            sha256: Some(Self::generate_sha256(&buffer)),
        })
    }
    fn generate_sha256(buffer: &[u8]) -> String {
        nr_core::utils::sha256::encode_to_string(buffer)
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
    /// Extra data that can be stored in the meta file. This is usually handled by the repository.
    ///
    /// Examples:
    /// - Version
    /// - Author
    /// - License
    #[serde(default)]
    pub extras: HashMap<String, Value>,
}
impl FileMeta {
    #[instrument(skip(path))]
    pub fn get_or_create_local(path: impl AsRef<Path>) -> Result<FileMeta, io::Error> {
        let path = path.as_ref();
        let meta_path = path.with_extension("nr-meta");
        debug!(?meta_path, ?path, "Creating or Loading Meta File");
        let meta = if meta_path.exists() {
            debug!("Meta File Exists");
            let file = File::open(meta_path)?;
            let meta: FileMeta = serde_json::from_reader(file)?;
            meta
        } else {
            let hashes = FileHashes::generate_from_path(&path)?;
            let (created, modified) = {
                let file = File::open(&path)?;
                let metadata = file.metadata()?;
                let modified = metadata.modified_as_chrono_or_now()?;
                let created = metadata.created_as_chrono_or_now()?;
                (created, modified)
            };

            let meta = FileMeta {
                hashes,
                created,
                modified,
                extras: HashMap::new(),
            };
            debug!(?meta, ?meta_path, "Writing Meta File");
            let file = File::create(meta_path)?;
            serde_json::to_writer(file, &meta)?;
            meta
        };
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

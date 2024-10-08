use std::{fmt::Debug, fs::File, io, path::Path};

use crate::{is_hidden_file, FileMeta};

use super::{utils::MetadataUtils, FileHashes, SerdeMime, StorageFileReader};
use chrono::{DateTime, FixedOffset, Local};

use nr_core::storage::FileTypeCheck;
use serde::{Deserialize, Serialize};

use strum::EnumIs;
use tracing::{debug, instrument};

/// Two types of files can be returned from the storage. A Directory or a File.
#[derive(EnumIs)]
pub enum StorageFile {
    /// A Directory will contain a list of files.
    Directory {
        meta: StorageFileMeta,
        files: Vec<StorageFileMeta>,
    },
    /// A File will contain the file meta and the file reader.
    File {
        meta: StorageFileMeta,
        content: StorageFileReader,
    },
}
impl FileTypeCheck for StorageFile {
    fn is_file(&self) -> bool {
        matches!(self, StorageFile::File { .. })
    }
    fn is_directory(&self) -> bool {
        matches!(self, StorageFile::Directory { .. })
    }
}

impl StorageFile {
    pub fn meta(&self) -> &StorageFileMeta {
        match self {
            StorageFile::Directory { meta, .. } => meta,
            StorageFile::File { meta, .. } => meta,
        }
    }
    pub fn file(self) -> Option<(StorageFileReader, StorageFileMeta)> {
        match self {
            StorageFile::File { content, meta } => Some((content, meta)),
            _ => None,
        }
    }
}
impl Debug for StorageFile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StorageFile::Directory { meta, files } => f
                .debug_struct("StorageFile::Directory")
                .field("meta", meta)
                .field("files", files)
                .finish(),
            StorageFile::File { meta, content } => f
                .debug_struct("StorageFile::File")
                .field("meta", meta)
                .field("content", &content)
                .finish(),
        }
    }
}
/// Publicly available meta data for a file.
#[derive(Serialize, Clone, Debug)]
#[non_exhaustive]
pub struct StorageFileMeta {
    /// File Name
    pub name: String,
    /// Individually stored file type. Different data is stored based if it is a file or a directory.
    pub file_type: FileType,
    /// Last time it was modified.
    pub modified: DateTime<FixedOffset>,
    /// The first time it was created.
    pub created: DateTime<FixedOffset>,
}
impl FileTypeCheck for StorageFileMeta {
    fn is_file(&self) -> bool {
        self.file_type.is_file()
    }
    fn is_directory(&self) -> bool {
        self.file_type.is_directory()
    }
}
impl StorageFileMeta {
    #[instrument(name = "StorageFileMeta::new_from_file", skip(path))]
    pub fn new_from_file(path: impl AsRef<Path>) -> Result<Self, io::Error> {
        let path = path.as_ref();
        debug!(?path, "Reading File Meta");
        let file = File::open(path)?;
        let metadata = file.metadata()?;
        // TODO: If the data is not available in the metadata. We should pull from the .nr-meta file.
        let modified = metadata
            .modified_as_chrono()?
            .unwrap_or_else(|| Local::now().into());
        let created = metadata
            .created_as_chrono()?
            .unwrap_or_else(|| Local::now().into());

        let file_type = if metadata.is_dir() {
            let mut file_count = 0;
            for entry in std::fs::read_dir(path)? {
                let entry = entry?;
                let path = entry.path();
                // Ignore hidden files.
                if is_hidden_file(&path) {
                    debug!(?path, "Skipping Meta File");
                    continue;
                }
                file_count += 1;
            }
            FileType::Directory { file_count }
        } else {
            let mime = super::utils::mime_type_for_file(&file, path.to_path_buf());
            let meta = FileMeta::get_or_create_local(path)?;

            FileType::File {
                file_size: metadata.len(),
                mime_type: mime,
                file_hash: meta.hashes,
            }
        };

        let name = path
            .file_name()
            .and_then(|v| v.to_str())
            .map(str::to_owned)
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    "Could not get file name from path",
                )
            })?;
        Ok(StorageFileMeta {
            name,
            file_type,
            modified,
            created,
        })
    }
    #[inline(always)]
    pub fn is_file(&self) -> bool {
        self.file_type.is_file()
    }
    #[inline(always)]
    pub fn is_directory(&self) -> bool {
        self.file_type.is_directory()
    }
}

impl StorageFileMeta {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn file_type(&self) -> &FileType {
        &self.file_type
    }
    pub fn modified(&self) -> &DateTime<FixedOffset> {
        &self.modified
    }
    pub fn created(&self) -> &DateTime<FixedOffset> {
        &self.created
    }
    pub fn file_extension(&self) -> Option<&str> {
        match self.file_type {
            FileType::File { .. } => self.name.split('.').last(),
            _ => None,
        }
    }
}
#[derive(Serialize, Deserialize, Clone, Debug, EnumIs)]
pub enum FileType {
    File {
        file_size: u64,
        mime_type: Option<SerdeMime>,
        file_hash: FileHashes,
    },
    Directory {
        file_count: u64,
    },
}

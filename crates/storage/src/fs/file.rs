use std::{
    fmt::Debug,
    fs::File,
    io::{self, Read},
    path::Path,
    pin::Pin,
};

use crate::FileMeta;

use super::{utils::MetadataUtils, FileHashes, SerdeMime};
use chrono::{DateTime, FixedOffset, Local};
use serde::{Deserialize, Serialize};
use strum::AsRefStr;
use tracing::{debug, instrument};

/// StorageFileReader is a wrapper around different types of readers.
#[derive(AsRefStr)]
pub enum StorageFileReader {
    /// File Readers will be the most common type of reader.
    /// For this reason, we will give it a special variant. To prevent dynamic dispatch.
    File(File),
    /// A Sync Reader type. This won't be used. As local will just use the File variant.
    Reader(Box<dyn std::io::Read + Send>),
    /// An Async Reader type. This will be used for remote storage. Such as S3.
    AsyncReader(Pin<Box<dyn tokio::io::AsyncRead + Send>>),
}
impl From<File> for StorageFileReader {
    fn from(file: File) -> Self {
        StorageFileReader::File(file)
    }
}
impl StorageFileReader {
    pub async fn read_to_end(self, buffer: &mut Vec<u8>) -> Result<usize, std::io::Error> {
        use tokio::io::AsyncReadExt;
        let size = match self {
            StorageFileReader::File(mut file) => Read::read_to_end(&mut file, buffer)?,
            StorageFileReader::Reader(mut reader) => Read::read_to_end(&mut reader, buffer)?,
            StorageFileReader::AsyncReader(mut reader) => {
                AsyncReadExt::read_to_end(&mut reader, buffer).await?
            }
        };
        Ok(size)
    }
    // TODO: Implement Streaming data from the reader to the response
}
/// Two types of files can be returned from the storage. A Directory or a File.
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
impl Debug for StorageFile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StorageFile::Directory { meta, files } => f
                .debug_struct("StorageFile::Directory")
                .field("meta", meta)
                .field("files", files)
                .finish(),
            StorageFile::File { meta, content } => {
                let read_type: &str = content.as_ref();
                f.debug_struct("StorageFile::File")
                    .field("meta", meta)
                    .field("content", &read_type)
                    .finish()
            }
        }
    }
}
/// Publicly available meta data for a file.
#[derive(Serialize, Clone, Debug)]
#[non_exhaustive]
pub struct StorageFileMeta {
    /// File Name
    name: String,
    /// Individually stored file type. Different data is stored based if it is a file or a directory.
    file_type: FileType,
    /// Last time it was modified.
    modified: DateTime<FixedOffset>,
    /// The first time it was created.
    created: DateTime<FixedOffset>,
}
impl StorageFileMeta {
    #[instrument(skip(path))]
    pub fn new_from_file(path: impl AsRef<Path>) -> Result<Self, io::Error> {
        let path = path.as_ref();
        debug!(?path, "Creating StorageFileMeta from file");
        let file = File::open(&path)?;
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
                if path.is_file() && path.extension().unwrap_or_default() == "nr-meta" {
                    debug!(?path, "Skipping Meta File");
                    // Check if file is a meta file
                    continue;
                }
                file_count += 1;
            }
            FileType::Directory {
                file_count: file_count,
            }
        } else {
            let mime = super::utils::mime_type_for_file(&file, path.to_path_buf());
            let meta = FileMeta::get_or_create_local(path.to_path_buf())?;

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
#[derive(Serialize, Deserialize, Clone, Debug)]
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

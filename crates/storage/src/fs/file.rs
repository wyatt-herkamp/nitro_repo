use std::{fmt::Debug, fs::File, io, path::Path};

use crate::{is_hidden_file, local::error::LocalStorageError, FileMeta};

use super::{
    FileHashes, SerdeMime, StorageFileReader,
};
use chrono::{DateTime, FixedOffset};

use derive_more::derive::From;
use nr_core::storage::FileTypeCheck;
use serde::{Deserialize, Serialize};

use strum::EnumIs;
use tracing::{debug, instrument};

/// Two types of files can be returned from the storage. A Directory or a File.
#[derive(EnumIs)]
pub enum StorageFile {
    /// A Directory will contain a list of files.
    Directory {
        meta: StorageFileMeta<DirectoryFileType>,
        files: Vec<StorageFileMeta<FileType>>,
    },
    /// A File will contain the file meta and the file reader.
    File {
        meta: StorageFileMeta<FileFileType>,
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
    pub fn file(self) -> Option<(StorageFileReader, StorageFileMeta<FileFileType>)> {
        match self {
            StorageFile::File { content, meta } => Some((content, meta)),
            _ => None,
        }
    }
    pub fn directory(
        self,
    ) -> Option<(
        Vec<StorageFileMeta<FileType>>,
        StorageFileMeta<DirectoryFileType>,
    )> {
        match self {
            StorageFile::Directory { files, meta } => Some((files, meta)),
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
pub struct StorageFileMeta<FT> {
    /// File Name
    pub name: String,
    /// Individually stored file type. Different data is stored based if it is a file or a directory.
    pub file_type: FT,
    /// Last time it was modified.
    pub modified: DateTime<FixedOffset>,
    /// The first time it was created.
    pub created: DateTime<FixedOffset>,
}
impl<FT> FileTypeCheck for StorageFileMeta<FT>
where
    FT: FileTypeCheck,
{
    fn is_file(&self) -> bool {
        self.file_type.is_file()
    }
    fn is_directory(&self) -> bool {
        self.file_type.is_directory()
    }
}

impl StorageFileMeta<FileType> {
    pub fn read_from_path(path: impl AsRef<Path>) -> Result<Self, LocalStorageError> {
        let path = path.as_ref();
        if path.is_dir() {
            return Ok(StorageFileMeta::read_from_directory(path)?.map_type(FileType::Directory));
        }
        Ok(StorageFileMeta::read_from_file(path)?.map_type(FileType::File))
    }
}

impl StorageFileMeta<DirectoryFileType> {
    #[instrument(name = "StorageFileMeta::new_file_meta", skip(path))]
    pub fn read_from_directory(path: impl AsRef<Path>) -> Result<Self, LocalStorageError> {
        let path = path.as_ref();
        if path.is_file() {
            return Err(LocalStorageError::ExpectedDirectory);
        }

        let file_count = path
            .read_dir()?
            .filter_map(|entry| {
                let entry = entry.ok()?;
                if is_hidden_file(&entry.path()) {
                    return None;
                }
                Some(entry)
            })
            .count() as u64;
        let file_meta: FileMeta = FileMeta::get_or_create_local(path)?;

        Ok(StorageFileMeta {
            name: get_file_name(path)?,
            file_type: DirectoryFileType { file_count },
            modified: file_meta.modified,
            created: file_meta.created,
        })
    }
}
impl StorageFileMeta<FileFileType> {
    #[instrument(name = "StorageFileMeta::new_file_meta", skip(path))]
    pub fn read_from_file(path: impl AsRef<Path>) -> Result<Self, LocalStorageError> {
        let path = path.as_ref();
        if path.is_dir() {
            return Err(LocalStorageError::ExpectedFile);
        }
        debug!(?path, "Reading File Meta");
        let file = File::open(path)?;

        let file_meta: FileMeta = FileMeta::get_or_create_local(path)?;
        let metadata = file.metadata()?;
        let file_type = {
            let mime = super::utils::mime_type_for_file(&file, path.to_path_buf());
            let meta = FileMeta::get_or_create_local(path)?;
            FileFileType {
                file_size: metadata.len(),
                mime_type: mime,
                file_hash: meta.hashes.unwrap_or_default(),
            }
            .into()
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
            modified: file_meta.modified,
            created: file_meta.created,
        })
    }
}

impl<FT> StorageFileMeta<FT> {
    fn map_type<T>(self, f: impl FnOnce(FT) -> T) -> StorageFileMeta<T> {
        StorageFileMeta {
            name: self.name,
            file_type: f(self.file_type),
            modified: self.modified,
            created: self.created,
        }
    }
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn file_type(&self) -> &FT {
        &self.file_type
    }
    pub fn modified(&self) -> &DateTime<FixedOffset> {
        &self.modified
    }
    pub fn created(&self) -> &DateTime<FixedOffset> {
        &self.created
    }
}
impl StorageFileMeta<FileFileType> {
    pub fn file_extension(&self) -> Option<&str> {
        self.name.split('.').last()
    }
}
#[derive(Serialize, Deserialize, Clone, Debug, EnumIs, From)]
pub enum FileType {
    File(FileFileType),
    Directory(DirectoryFileType),
}
impl FileTypeCheck for FileType {
    fn is_file(&self) -> bool {
        matches!(self, FileType::File(_))
    }
    fn is_directory(&self) -> bool {
        matches!(self, FileType::Directory(_))
    }
}
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FileFileType {
    pub file_size: u64,
    pub mime_type: Option<SerdeMime>,
    pub file_hash: FileHashes,
}
impl FileTypeCheck for FileFileType {
    fn is_file(&self) -> bool {
        true
    }
    fn is_directory(&self) -> bool {
        false
    }
}
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DirectoryFileType {
    pub file_count: u64,
}
impl FileTypeCheck for DirectoryFileType {
    fn is_file(&self) -> bool {
        false
    }
    fn is_directory(&self) -> bool {
        true
    }
}

fn get_file_name(path: &Path) -> Result<String, io::Error> {
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
    Ok(name)
}

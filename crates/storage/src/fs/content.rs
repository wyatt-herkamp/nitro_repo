use std::{io::Write, path::PathBuf};

use bytes::Bytes;
use derive_more::derive::From;

use super::FileHashes;
/// FileContent is a enum that can be used to represent the content of a file.
///
/// This is used from copying files from a request to a storage
#[derive(Debug, Clone)]
pub enum FileContent {
    Path(PathBuf),
    Content(Vec<u8>),
    Bytes(Bytes),
}
impl<B: AsRef<[u8]>> From<B> for FileContent {
    fn from(bytes: B) -> Self {
        FileContent::Content(bytes.as_ref().to_vec())
    }
}
#[derive(Debug, From)]
pub enum FileContentBytes {
    Content(Vec<u8>),
    Bytes(Bytes),
}
impl From<FileContentBytes> for Vec<u8> {
    fn from(bytes: FileContentBytes) -> Self {
        match bytes {
            FileContentBytes::Content(content) => content,
            FileContentBytes::Bytes(bytes) => bytes.into_iter().collect(),
        }
    }
}
impl FileContentBytes {
    pub fn len(&self) -> usize {
        match self {
            FileContentBytes::Content(content) => content.len(),
            FileContentBytes::Bytes(bytes) => bytes.len(),
        }
    }
}
impl AsRef<[u8]> for FileContentBytes {
    fn as_ref(&self) -> &[u8] {
        match self {
            FileContentBytes::Content(content) => content.as_ref(),
            FileContentBytes::Bytes(bytes) => bytes.as_ref(),
        }
    }
}
impl TryFrom<FileContent> for FileContentBytes {
    type Error = std::io::Error;
    fn try_from(value: FileContent) -> Result<Self, Self::Error> {
        match value {
            FileContent::Path(path) => {
                let bytes = std::fs::read(path)?;
                Ok(FileContentBytes::Content(bytes))
            }
            FileContent::Content(content) => Ok(FileContentBytes::Content(content)),
            FileContent::Bytes(bytes) => Ok(FileContentBytes::Bytes(bytes)),
        }
    }
}
impl TryFrom<FileContent> for Vec<u8> {
    type Error = std::io::Error;
    fn try_from(value: FileContent) -> Result<Self, Self::Error> {
        match value {
            FileContent::Path(path) => {
                let bytes = std::fs::read(path)?;
                Ok(bytes)
            }
            FileContent::Content(content) => Ok(content),
            FileContent::Bytes(bytes) => Ok(bytes.into_iter().collect()),
        }
    }
}
impl FileContent {
    pub fn generate_hashes(&self) -> std::io::Result<FileHashes> {
        let bytes = match self {
            FileContent::Path(path) => FileHashes::generate_from_path(path)?,
            FileContent::Content(content) => FileHashes::generate_from_bytes(content),
            FileContent::Bytes(bytes) => FileHashes::generate_from_bytes(bytes),
        };
        Ok(bytes)
    }
    pub fn write_to(&self, writer: &mut impl Write) -> std::io::Result<usize> {
        let bytes = match self {
            FileContent::Path(path) => {
                let mut file = std::fs::File::open(path)?;
                std::io::copy(&mut file, writer)? as usize
            }
            FileContent::Content(content) => {
                writer.write_all(content)?;
                content.len()
            }
            FileContent::Bytes(bytes) => {
                writer.write_all(bytes)?;
                bytes.len()
            }
        };
        Ok(bytes)
    }
}

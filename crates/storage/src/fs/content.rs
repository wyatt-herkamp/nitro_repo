use std::{io::Write, path::PathBuf};

use bytes::Bytes;
use derive_more::derive::From;
/// FileContent is a enum that can be used to represent the content of a file.
///
/// This is used from copying files from a request to a storage
#[derive(Debug, From)]
pub enum FileContent {
    Path(PathBuf),
    Content(Vec<u8>),
    Bytes(Bytes),
}

impl FileContent {
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

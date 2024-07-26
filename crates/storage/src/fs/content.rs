use std::{io::Write, path::PathBuf};
/// FileContent is a enum that can be used to represent the content of a file.
///
/// This is used from copying files from a request to a storage
#[derive(Debug)]
pub enum FileContent {
    Path(PathBuf),
    Content(Vec<u8>),
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
        };
        Ok(bytes)
    }
}

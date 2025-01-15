use std::path::{Path, PathBuf};

use thiserror::Error;
use tracing::instrument;

#[derive(Debug, Error)]
#[error("Parent directory for {0} does not exist")]
pub struct ParentDirectoryDoesNotExist(pub PathBuf);

#[derive(Debug, Error)]
pub enum ExtensionError {
    #[error("The extension of path {0} is not UTF-8")]
    ExtensionNotUtf8(PathBuf),
}

pub trait PathUtils {
    fn parent_or_err(&self) -> Result<&Path, ParentDirectoryDoesNotExist>;
    /// Appends an extension to the path.
    fn add_extension(&self, extension: &str) -> Result<PathBuf, ExtensionError>;
    /// Gets the current extension and attempts to convert it to a string.
    fn extension_to_string(&self) -> Result<Option<&str>, ExtensionError>;
}
impl PathUtils for PathBuf {
    fn parent_or_err(&self) -> Result<&Path, ParentDirectoryDoesNotExist> {
        self.parent()
            .ok_or_else(|| ParentDirectoryDoesNotExist(self.clone()))
    }
    fn extension_to_string(&self) -> Result<Option<&str>, ExtensionError> {
        self.extension()
            .map(|v| {
                v.to_str()
                    .ok_or_else(|| ExtensionError::ExtensionNotUtf8(self.clone()))
            })
            .transpose()
    }
    #[instrument]
    fn add_extension(&self, extension: &str) -> Result<PathBuf, ExtensionError> {
        let mut path = self.clone();
        let old_extension = path.extension_to_string()?;
        match old_extension {
            Some(old_extension) => {
                path.set_extension(format!("{}.{}", old_extension, extension));
            }
            None => {
                path.set_extension(extension);
            }
        }
        Ok(path)
    }
}

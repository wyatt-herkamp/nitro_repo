use std::path::PathBuf;

use thiserror::Error;

#[derive(Debug, Error)]
#[error("Parent directory for {0} does not exist")]
#[allow(dead_code)]
pub struct ParentDirectoryDoesNotExist(pub PathBuf);

use crate::error::internal_error::InternalError;
use regex::internal::Input;
use std::path::{Path, PathBuf};
use tokio::fs::{create_dir_all, remove_file, File};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

pub mod markdown;
#[derive(Clone, Debug)]
pub struct GeneratorCache {
    pub local_path: PathBuf,
}
impl GeneratorCache {
    pub async fn get_as_string(
        &self,
        file: impl AsRef<Path>,
    ) -> Result<Option<String>, InternalError> {
        let path = self.local_path.join(file.as_ref());
        if !path.exists() {
            return Ok(None);
        }
        let mut file = File::open(path).await?;
        let mut contents = String::new();
        file.read_to_string(&mut contents).await?;
        Ok(Some(contents))
    }
    pub async fn get_as_bytes(
        &self,
        file: impl AsRef<Path>,
    ) -> Result<Option<Vec<u8>>, InternalError> {
        let path = self.local_path.join(file.as_ref());
        if !path.exists() {
            return Ok(None);
        }
        let mut file = File::open(path).await?;
        let mut contents = Vec::new();
        file.read_to_end(&mut contents).await?;
        Ok(Some(contents))
    }
    pub async fn push_to_cache(
        &self,
        file: impl AsRef<Path>,
        contents: impl AsRef<[u8]>,
    ) -> Result<(), InternalError> {
        let path = self.local_path.join(file.as_ref());
        if path.exists() {
            remove_file(&path).await?;
        } else {
            let x = path
                .parent()
                .ok_or(InternalError::Error("Failed to get parent".to_string()))?;
            create_dir_all(x).await?;
        }
        let mut file = File::create(path).await?;
        file.write_all(contents.as_ref()).await?;
        Ok(())
    }
}

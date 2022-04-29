use std::collections::HashMap;
use std::path::Path;

use tokio::fs;
use tokio::fs::{read_to_string, OpenOptions};

use crate::storage::models::{Storage, StorageFile, STORAGE_FILE, STORAGE_FILE_BAK};
use crate::storage::StorageHandlerType;
use async_trait::async_trait;
use serde::{Serialize, Serializer};
use thiserror::Error;
use tokio::io::AsyncWriteExt;
use tokio::sync::RwLock;

#[derive(Error, Debug)]
pub enum MultiStorageError {
    #[error("IO error {0}")]
    IOError(std::io::Error),
    #[error("JSON error {0}")]
    JSONError(serde_json::Error),
}

impl From<std::io::Error> for MultiStorageError {
    fn from(err: std::io::Error) -> MultiStorageError {
        MultiStorageError::IOError(err)
    }
}

impl From<serde_json::Error> for MultiStorageError {
    fn from(err: serde_json::Error) -> MultiStorageError {
        MultiStorageError::JSONError(err)
    }
}

pub async fn load_storages() -> Result<HashMap<String, Storage>, MultiStorageError> {
    let path = Path::new(STORAGE_FILE);
    if !path.exists() {
        return Ok(HashMap::new());
    }
    let string = read_to_string(&path).await?;
    let result: HashMap<String, Storage> = serde_json::from_str(&string)?;
    Ok(result)
}

pub async fn save_storages(storages: &HashMap<String, Storage>) -> Result<(), MultiStorageError> {
    let result = serde_json::to_string(&storages)?;
    let path = Path::new(STORAGE_FILE);
    let bak = Path::new(STORAGE_FILE_BAK);
    if bak.exists() {
        fs::remove_file(bak).await?;
    }
    if path.exists() {
        fs::rename(path, bak).await?;
    }
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .open(path)
        .await?;
    file.write_all(result.as_bytes()).await?;
    Ok(())
}

pub struct MultiStorageController {
    pub storages: RwLock<HashMap<String, Storage>>,
}

impl MultiStorageController {
    pub async fn init() -> Result<MultiStorageController, MultiStorageError> {
        let result = load_storages().await?;
        Ok(MultiStorageController {
            storages: RwLock::new(result),
        })
    }
}

impl Serialize for MultiStorageController {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_some(self)
    }
}

#[async_trait]
impl StorageHandlerType for MultiStorageController {
    type Error = MultiStorageError;

    async fn get_storage_by_name(&self, name: &str) -> Result<Option<Storage>, Self::Error> {
        let storages = self.storages.read().await;
        return Ok(storages.get(name).cloned());
    }

    async fn create_storage(&self, storage: Storage) -> Result<(), Self::Error> {
        let mut storages = self.storages.write().await;
        //TODO Prepare Setup
        storages.insert(storage.config.name.clone(), storage);
        save_storages(&storages).await?;
        return Ok(());
    }

    async fn delete_storage(&self, storage: &str) -> Result<bool, Self::Error> {
        let mut storages = self.storages.write().await;
        let option = storages.remove(storage);
        if option.is_none() {
            return Ok(false);
        }
        save_storages(&storages).await?;
        //TODO Call Delete Functions
        return Ok(true);
    }

    async fn storages_as_file_list(&self) -> Result<Vec<StorageFile>, Self::Error> {
        let storages = self.storages.read().await;
        let mut files = Vec::new();
        for (name, storage) in storages.iter() {
            files.push(StorageFile {
                name: name.clone(),
                full_path: name.clone(),
                directory: true,
                file_size: 0,
                created: storage.config.created as u128,
            });
        }
        return Ok(files);
    }
}

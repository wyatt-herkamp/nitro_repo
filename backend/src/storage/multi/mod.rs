use std::collections::HashMap;
use std::path::{Path, PathBuf};

use log::{error, info};
use tokio::fs;
use tokio::fs::{read_to_string, OpenOptions};
use tokio::io::AsyncWriteExt;
use tokio::sync::{RwLock, RwLockReadGuard};

use crate::storage::bad_storage::BadStorage;
use crate::storage::error::StorageError;
use crate::storage::file::StorageFile;
use crate::storage::models::{
    Storage, StorageFactory, StorageSaver, StorageStatus, STORAGE_FILE, STORAGE_FILE_BAK,
};
use crate::storage::DynamicStorage;

pub mod web;

async fn load_storages(
    storages_file: PathBuf,
) -> Result<HashMap<String, DynamicStorage>, StorageError> {
    if !storages_file.exists() {
        return Ok(HashMap::new());
    }
    let string = read_to_string(&storages_file).await?;
    let result: Vec<StorageFactory> = serde_json::from_str(&string)?;
    let mut values: HashMap<String, DynamicStorage> = HashMap::new();
    for factory in result {
        let name = factory.generic_config.name.clone();
        let storage = match factory.build().await {
            Ok(value) => value,
            Err((error, factory)) => DynamicStorage::BadStorage(BadStorage::create(factory, error)),
        };
        values.insert(name, storage);
    }
    Ok(values)
}

pub async fn save_storages(storages: &Vec<StorageSaver>) -> Result<(), StorageError> {
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
#[derive(Debug)]
pub struct MultiStorageController {
    pub storages: RwLock<HashMap<String, DynamicStorage>>,
}

impl MultiStorageController {
    pub async fn init(storages_file: PathBuf) -> Result<MultiStorageController, StorageError> {
        info!("Loading Storages");
        let result = load_storages(storages_file).await?;
        let controller = MultiStorageController {
            storages: RwLock::new(result),
        };
        controller.load_unloaded_storages().await?;
        Ok(controller)
    }
}

impl MultiStorageController {
    pub async fn get_storage_by_name(
        &self,
        name: &str,
    ) -> Result<Option<RwLockReadGuard<DynamicStorage>>, StorageError> {
        let storages = self.storages.read().await;
        if !storages.contains_key(name) {
            return Ok(None);
        }
        let storage = RwLockReadGuard::map(storages, |storages| {
            let storage = storages.get(name).unwrap();
            storage
        });
        Ok(Some(storage))
    }
    pub async fn does_storage_exist(&self, name: &str) -> Result<bool, StorageError> {
        let storages = self.storages.read().await;
        Ok(storages.contains_key(name))
    }

    pub async fn create_storage<'a>(&self, storage: StorageFactory) -> Result<(), StorageError> {
        let mut storages = self.storages.write().await;
        let name = storage.generic_config.name.clone();

        if storages.contains_key(&name) {
            return Err(StorageError::StorageAlreadyExist);
        }
        {
            let mut storages_saving = Vec::new();
            for (_, storage) in storages.iter() {
                storages_saving.push(storage.config_for_saving());
            }
            storages_saving.push(storage.config_for_saving());
            save_storages(&storages_saving).await?;
        }
        let storage_name = name.clone();

        let storage_handler = storage.build().await.unwrap();
        storages.insert(storage_name.clone(), storage_handler);
        Ok(())
    }
    /// Attempts to run the storage load on any storages that are unloaded.
    /// This will include the Error storages
    pub async fn load_unloaded_storages<'a>(&self) -> Result<(), StorageError> {
        let mut guard = self.storages.write().await;
        for (name, storage) in guard.iter_mut() {
            match storage.status() {
                StorageStatus::Unloaded => {
                    info!("Loading Storage {}", name);
                    storage.load().await?;
                }
                StorageStatus::Error(_) => {
                    info!("Loading Storage {}", name);
                    storage.load().await?;
                }
                _ => {}
            }
        }
        Ok(())
    }

    pub async fn delete_storage(&self, storage: &str) -> Result<bool, StorageError> {
        let mut storages = self.storages.write().await;
        let option = storages.remove(storage);
        if option.is_none() {
            return Ok(false);
        }
        let mut storage = option.unwrap();
        drop(storages);
        // Save the new storages
        save_storages(self.storage_savers().await.as_ref()).await?;
        if let Err(error) = storage.unload() {
            error!(
                "Storage has been removed config. However, it failed to unload properly {}",
                error
            );
        }
        Ok(true)
    }
    pub async fn storage_savers(&self) -> Vec<StorageSaver> {
        let storages = self.storages.read().await;

        let mut storages_saving = Vec::new();
        for (_, storage) in storages.iter() {
            storages_saving.push(storage.config_for_saving().clone());
        }
        storages_saving
    }
    pub async fn names(&self) -> Vec<String> {
        let storages = self.storages.read().await;
        storages.keys().cloned().collect()
    }
    pub async fn storages_as_file_list(&self) -> Result<Vec<StorageFile>, StorageError> {
        let storages = self.storages.read().await;
        let mut files = Vec::new();
        for (name, storage) in storages.iter() {
            files.push(StorageFile {
                name: name.clone(),
                full_path: name.clone(),
                mime: "text/directory".to_string(),
                directory: true,
                file_size: 0,
                modified: 0,
                created: storage.config_for_saving().generic_config.created as u128,
            });
        }
        Ok(files)
    }
}

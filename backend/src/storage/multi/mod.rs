use std::collections::HashMap;
use std::path::Path;

use tokio::fs;
use tokio::fs::{read_to_string, OpenOptions};

use crate::storage::models::{
    Storage, StorageError, StorageFile, UnloadedStorage, STORAGE_FILE, STORAGE_FILE_BAK,
};

use serde::{Serialize, Serializer};

use tokio::io::AsyncWriteExt;
use tokio::sync::RwLock;

use crate::storage::handler::StorageHandler;

pub async fn load_storages() -> Result<HashMap<String, Storage>, StorageError> {
    let path = Path::new(STORAGE_FILE);
    if !path.exists() {
        return Ok(HashMap::new());
    }
    let string = read_to_string(&path).await?;
    let result: Vec<UnloadedStorage> = serde_json::from_str(&string)?;
    let mut values: HashMap<String, Storage> = HashMap::new();
    for unloaded_storage in result {
        let key = unloaded_storage.config.name.clone();
        let storage = Storage {
            storage_handler: crate::storage::handler::StorageHandler::load(
                unloaded_storage.storage_handler,
            )
            .await?,
            config: unloaded_storage.config,
        };

        values.insert(key, storage);
    }
    Ok(values)
}

pub async fn save_storages(storages: &HashMap<String, Storage>) -> Result<(), StorageError> {
    let mut values: Vec<UnloadedStorage> = Vec::new();
    for (_, storage) in storages {
        values.push(UnloadedStorage {
            storage_handler: storage.storage_handler.save_value()?,
            config: storage.config.clone(),
        })
    }
    let result = serde_json::to_string(&values)?;
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
    pub async fn init() -> Result<MultiStorageController, StorageError> {
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

impl MultiStorageController {
    pub async fn get_storage_by_name(&self, name: &str) -> Result<Option<Storage>, StorageError> {
        let storages = self.storages.read().await;
        return Ok(storages.get(name).cloned());
    }

    pub async fn create_storage(&self, storage: UnloadedStorage) -> Result<Storage, StorageError> {
        let storages = self.storages.read().await;
        let name = storage.config.name.clone();

        if storages.contains_key(&name) {
            return Err(StorageError::StorageAlreadyExist);
        }
        /// Return Storage Lock if not needed
        let mut saving_map = storages.clone();
        drop(storages);
        let storage = {
            //Initialize Storage
            let storage_handler = StorageHandler::load(storage.storage_handler).await?;
            let storage = Storage {
                storage_handler,
                config: storage.config,
            };
            saving_map.insert(name.clone(), storage);
            save_storages(&saving_map).await?;
            saving_map.remove(&name).ok_or(StorageError::LoadFailure(
                "Failed to create storage. Unable to find new data".to_string(),
            ))
        }?;
        let mut storages = self.storages.write().await;
        storages.insert(name.clone(), storage);
        return storages
            .get(&name)
            .cloned()
            .ok_or(StorageError::LoadFailure(
                "Failed to create storage. Unable to find new data".to_string(),
            ));
    }

    pub async fn delete_storage(&self, storage: &str) -> Result<bool, StorageError> {
        let mut storages = self.storages.write().await;
        let option = storages.remove(storage);
        if option.is_none() {
            return Ok(false);
        }
        save_storages(&storages).await?;
        //TODO Call Delete Functions
        Ok(true)
    }

    pub async fn storages_as_file_list(&self) -> Result<Vec<StorageFile>, StorageError> {
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
        Ok(files)
    }
}

use lockfree::map::Map;

use std::mem;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::repository::handler::DynamicRepositoryHandler;
use log::{error, info};
use tokio::fs;
use tokio::fs::{read_to_string, OpenOptions};
use tokio::io::AsyncWriteExt;

use crate::storage::bad_storage::BadStorage;
use crate::storage::error::StorageError;
use crate::storage::file::StorageFile;

use crate::storage::models::{Storage, STORAGE_FILE, STORAGE_FILE_BAK};
use crate::storage::{DynamicStorage, StorageSaver};

pub mod web;

async fn load_storages(
    storages_file: PathBuf,
) -> Result<Map<String, Arc<DynamicStorage>>, StorageError> {
    if !storages_file.exists() {
        return Ok(Map::new());
    }
    let string = read_to_string(&storages_file).await?;
    let result: Vec<StorageSaver> = serde_json::from_str(&string)?;
    let values: Map<String, Arc<DynamicStorage>> = Map::new();
    for factory in result {
        let name = factory.generic_config.id.clone();
        let storage = match DynamicStorage::new(factory).await {
            Ok(value) => value,
            Err((error, factory)) => DynamicStorage::BadStorage(BadStorage::create(factory, error)),
        };
        values.insert(name, Arc::new(storage));
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
pub struct MultiStorageController<S: Storage> {
    pub storages: Map<String, Arc<S>>,
    pub unloaded_storages: Map<String, Arc<S>>,
}

impl MultiStorageController<DynamicStorage> {
    pub async fn init(
        storages_file: PathBuf,
    ) -> Result<MultiStorageController<DynamicStorage>, StorageError> {
        info!("Loading Storages");
        let result = load_storages(storages_file).await?;
        let mut controller = MultiStorageController {
            storages: Map::new(),
            unloaded_storages: result,
        };
        controller.load_unloaded_storages().await?;
        Ok(controller)
    }
    pub async fn get_storage_by_name(
        &self,
        name: &str,
    ) -> Result<Option<Arc<DynamicStorage>>, StorageError> {
        let storages = self.storages.get(name);
        if let Some(storage) = storages {
            Ok(Some(storage.as_ref().1.clone()))
        } else {
            Ok(None)
        }
    }
    pub fn does_storage_exist(&self, name: &str) -> Result<bool, StorageError> {
        let storages = self.storages.get(name);
        Ok(storages.is_some())
    }

    /// Attempts to run the storage load on any storages that are unloaded.
    /// This will include the Error storages
    pub async fn load_unloaded_storages<'a>(&mut self) -> Result<(), StorageError> {
        let unloaded = mem::take(&mut self.unloaded_storages);
        for (name, storage) in unloaded.into_iter() {
            match storage.get_repos_to_load().await {
                Ok(repositories) => {
                    for (name, repository) in repositories.into_iter() {
                        info!("Loading repository {}", name);
                        let handler =
                            DynamicRepositoryHandler::new_dyn_storage(storage.clone(), repository)
                                .await
                                .map_err(|_error| {
                                    error!("Error loading repository {}", name);
                                });
                        if let Ok(handler) = handler {
                            storage.add_repo_loaded(handler)?;
                        }
                    }
                }
                Err(error) => {
                    error!("Error loading storage {}: {}", name, error);
                }
            }
            self.storages.insert(name, storage);
        }
        Ok(())
    }

    pub async fn create_storage<'a>(&self, _storage: StorageSaver) -> Result<(), StorageError> {
        todo!()
    }

    pub async fn delete_storage(&self, _storage: &str) -> Result<bool, StorageError> {
        todo!()
    }
    pub async fn storage_savers(&self) -> Vec<StorageSaver> {
        todo!()
    }
    pub async fn names(&self) -> Vec<String> {
        self.storages
            .iter()
            .map(|v| v.key().clone())
            .collect::<Vec<_>>()
    }
    pub async fn storages_as_file_list(&self) -> Result<Vec<StorageFile>, StorageError> {
        let mut files = Vec::new();
        for v in self.storages.iter() {
            let name = v.0.clone();
            let create = v.1.as_ref().storage_config().generic_config.created;
            files.push(StorageFile {
                name: name.clone(),
                full_path: name.clone(),
                mime: "text/directory".to_string(),
                directory: true,
                file_size: 0,
                modified: 0,
                created: create as u128,
            });
        }
        Ok(files)
    }
}

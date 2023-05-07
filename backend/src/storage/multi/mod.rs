use lockfree::map::Map;

use ahash::AHashMap;
use std::mem;
use std::path::PathBuf;
use std::sync::Arc;

use crate::repository::handler::DynamicRepositoryHandler;
use log::{error, info};
use serde::Deserialize;

use tokio::fs::{read_to_string, OpenOptions};
use tokio::io::AsyncWriteExt;
use tokio::sync::RwLock;

use crate::storage::bad_storage::BadStorage;
use crate::storage::error::StorageError;
use crate::storage::file::StorageFile;

use crate::storage::error::StorageError::StorageCreateError;
use crate::storage::models::Storage;
use crate::storage::{DynamicStorage, StorageSaver};

pub mod web;

async fn load_storages(
    storages_file: &PathBuf,
) -> Result<AHashMap<String, Arc<DynamicStorage>>, StorageError> {
    if !storages_file.exists() {
        return Ok(AHashMap::default());
    }
    let string = read_to_string(&storages_file).await?;
    let result: Vec<StorageSaver> = serde_json::from_str(&string)?;
    let mut values: AHashMap<String, Arc<DynamicStorage>> = AHashMap::new();
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

pub async fn save_storages(
    storages: Vec<StorageSaver>,
    storages_file: &PathBuf,
) -> Result<(), StorageError> {
    let result = serde_json::to_string(&storages)?;
    let mut file = OpenOptions::new()
        .write(true)
        .append(false)
        .create(true)
        .open(storages_file)
        .await?;
    file.write_all(result.as_bytes()).await?;
    Ok(())
}
type Storages<S> = RwLock<AHashMap<String, Arc<S>>>;
#[derive(Debug)]
pub struct MultiStorageController<S: Storage> {
    pub storages: Storages<S>,
    pub unloaded_storages: Storages<S>,
    pub storage_file: PathBuf,
}
#[derive(Debug, Deserialize)]
pub enum PurgeLevel {
    All,
    Configs,
    RemoveFromList,
}
impl MultiStorageController<DynamicStorage> {
    pub async fn init(
        storages_file: PathBuf,
    ) -> Result<MultiStorageController<DynamicStorage>, StorageError> {
        info!("Loading Storages");
        let result = load_storages(&storages_file).await?;
        let mut controller = MultiStorageController {
            storages: RwLock::new(AHashMap::new()),
            unloaded_storages: RwLock::new(result),
            storage_file: storages_file,
        };
        controller.load_unloaded_storages().await?;
        Ok(controller)
    }
    pub async fn get_storage_by_name(&self, name: &str) -> Option<Arc<DynamicStorage>> {
        self.storages.read().await.get(name).map(|x| x.clone())
    }
    pub async fn does_storage_exist(&self, name: &str) -> Result<bool, StorageError> {
        Ok(self.storages.read().await.contains_key(name))
    }

    /// Attempts to run the storages load on any storages that are unloaded.
    /// This will include the Error storages
    pub async fn load_unloaded_storages<'a>(&mut self) -> Result<(), StorageError> {
        let unloaded = mem::take(self.unloaded_storages.get_mut());
        for (name, storage) in unloaded.into_iter() {
            match storage.get_repos_to_load().await {
                Ok(repositories) => {
                    for (name, repository) in repositories.into_iter() {
                        info!("Loading repository {}", name);
                        let handler =
                            DynamicRepositoryHandler::new_dyn_storage(storage.clone(), repository)
                                .await
                                .map_err(|error| {
                                    error!("Error loading repository {}. Error {}", name, error);
                                });
                        if let Ok(handler) = handler {
                            storage.add_repo_loaded(handler)?;
                        }
                    }
                }
                Err(error) => {
                    error!("Error loading storages {}: {}", name, error);
                }
            }
            self.storages.get_mut().insert(name, storage);
        }
        Ok(())
    }

    /// Starts by checking all the storages to see if the name already exists
    /// Collects all the StorageSavers into an Array.
    /// Checks to ensure the storages will load correctly. If it will not it will error our
    /// Saves the new storages config
    /// Adds the storages to the main Storage map without loading repositories. Because its a new storages
    pub async fn create_storage<'a>(&self, mut storage: StorageSaver) -> Result<(), StorageError> {
        storage.generic_config.id = storage.generic_config.id.to_lowercase();
        let name = storage.generic_config.id.clone();
        // Check if the storages already exists then collect all Vec<StorageSaver> and add the new one
        let mut storages = Vec::new();
        let storage_ref = self.storages.read().await;
        for (storage_name, value) in storage_ref.iter() {
            if storage_name.eq(&name) {
                return Err(StorageCreateError("Storage already exists".to_string()));
            }
            storages.push(value.storage_config().clone());
        }
        let storage = DynamicStorage::create_new(storage)
            .await
            .map_err(|(error, v)| {
                error!("Error creating storages {:?}.", v);
                StorageCreateError(error.to_string())
            })?;
        drop(storage_ref);
        storages.push(storage.storage_config().clone());
        save_storages(storages, &self.storage_file).await?;
        info!("Created storage {}", name);
        info!("Loading storage {:?}", storage);
        self.storages.write().await.insert(name, Arc::new(storage));
        info!("Loaded storage");
        Ok(())
    }

    /// Follows the same steps as create_storage but will treat the new storages as something that has data in it.
    pub async fn recover_storage(&self, storage: StorageSaver) -> Result<(), StorageError> {
        let name = storage.generic_config.id.clone();
        // Check if the storages already exists then collect all Vec<StorageSaver> and add the new one

        let storages_ref = self.storages.read().await;
        let mut storages = Vec::with_capacity(storages_ref.len() + 1);
        for (storage_name, storage) in storages_ref.iter() {
            if storage_name.eq(&name) {
                return Err(StorageCreateError("Storage already exists".to_string()));
            }
            storages.push(storage.storage_config().clone());
        }
        let storage = DynamicStorage::create_new(storage)
            .await
            .map_err(|(error, v)| {
                error!("Error creating storages {:?}.", v);
                StorageCreateError(error.to_string())
            })
            .map(Arc::new)?;
        let repositories = storage.get_repos_to_load().await?;
        for (name, repository) in repositories.into_iter() {
            info!("Loading repository {} From Recovery", name);
            let handler = DynamicRepositoryHandler::new_dyn_storage(storage.clone(), repository)
                .await
                .map_err(|error| {
                    error!("Error loading repository {}. Error {}", name, error);
                });
            if let Ok(handler) = handler {
                storage.add_repo_loaded(handler)?;
            }
        }

        storages.push(storage.storage_config().clone());
        save_storages(storages, &self.storage_file).await?;

        self.storages.write().await.insert(name, storage);
        Ok(())
    }

    pub async fn delete_storage(
        &self,
        storage: impl AsRef<str>,
        purge_level: PurgeLevel,
    ) -> Result<(), StorageError> {
        let option = self
            .storages
            .write()
            .await
            .remove(storage.as_ref())
            .ok_or_else(|| {
                StorageError::StorageDeleteError("Storage does not exist".to_string())
            })?;
        save_storages(self.storage_savers().await, &self.storage_file).await?;

        match purge_level {
            PurgeLevel::All => {
                let x = option.get_repository_list()?;
                for repository in x.into_iter() {
                    if let Err(error) = option.delete_repository(&repository.name, true).await {
                        error!(
                            "Error deleting repository {}. Error {}",
                            repository.name, error
                        );
                    }
                }
            }
            PurgeLevel::Configs => {
                let x = option.get_repository_list()?;
                for repository in x.into_iter() {
                    if let Err(error) = option.delete_repository(&repository.name, false).await {
                        error!(
                            "Error deleting repository {}. Error {}",
                            repository.name, error
                        );
                    }
                }
            }
            _ => {}
        }

        Ok(())
    }

    pub async fn storage_savers(&self) -> Vec<StorageSaver> {
        let mut result = Vec::new();
        let storage_ref = self.storages.read().await;
        for val in storage_ref.values() {
            let saver = val.storage_config().clone();
            result.push(saver);
        }
        result
    }
    pub async fn names(&self) -> Vec<String> {
        self.storages.read().await.keys().cloned().collect()
    }
    pub async fn storages_as_file_list(&self) -> Result<Vec<StorageFile>, StorageError> {
        let storages_ref = self.storages.read().await;
        let mut files = Vec::with_capacity(storages_ref.len());
        for (key, value) in storages_ref.iter() {
            let name = key.clone();
            let create = value.storage_config().generic_config.created;
            files.push(StorageFile {
                name: name.clone(),
                full_path: name.clone(),
                mime: "text/directory".to_string(),
                directory: true,
                file_size: 0,
                modified: None,
                created: create,
            });
        }
        Ok(files)
    }
}

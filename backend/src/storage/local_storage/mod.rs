use crate::repository::REPOSITORY_CONF_FOLDER;
use crate::storage::models::{
    Storage, StorageConfig, StorageFactory, StorageSaver, StorageStatus, StorageType,
};
use crate::storage::STORAGE_CONFIG;
use crate::utils::get_current_time;

use log::{debug, trace, warn};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use std::path::PathBuf;



use crate::repository::data::{RepositoryConfig, RepositoryType};
use crate::storage::error::StorageError;
use crate::storage::error::StorageError::RepositoryMissing;
use async_trait::async_trait;

use serde_json::Value;

use crate::storage::file::{StorageDirectoryResponse, StorageFile, StorageFileResponse};
use tokio::fs::{create_dir, create_dir_all, read_to_string, remove_dir, remove_file, OpenOptions};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::{RwLock, RwLockReadGuard};

#[derive(Debug)]
pub struct LocalStorage {
    pub config: LocalConfig,
    pub storage_config: StorageConfig,
    pub status: StorageStatus,
    pub repositories: RwLock<HashMap<String, RepositoryConfig>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalConfig {
    pub location: String,
}

pub struct LocalFile {
    pub path: PathBuf,
}

impl LocalStorage {
    pub fn get_storage_folder(&self) -> PathBuf {
        PathBuf::from(&self.config.location)
    }
    pub fn get_repository_folder(&self, repository: &str) -> PathBuf {
        self.get_storage_folder().join(repository)
    }
}

impl LocalStorage {
    async fn load_repositories(
        path: PathBuf,
    ) -> Result<HashMap<String, RepositoryConfig>, StorageError> {
        if !path.exists() {
            return Ok(HashMap::new());
        }
        let string = read_to_string(&path).await?;
        let result: Vec<RepositoryConfig> = serde_json::from_str(&string)?;
        let mut values = HashMap::new();
        for x in result {
            values.insert(x.name.clone(), x);
        }
        Ok(values)
    }
    async fn save_repositories(&self) -> Result<(), StorageError> {
        let repositories = self.repositories.read().await;
        let conf = self.get_storage_folder().join(STORAGE_CONFIG);

        let file = OpenOptions::new()
            .create(true)
            .open(conf)
            .await?
            .into_std()
            .await;
        let values: Vec<&RepositoryConfig> = repositories.values().collect();
        serde_json::to_writer_pretty(file, &values)?;
        Ok(())
    }
}

#[async_trait]
impl Storage for LocalStorage {
    fn new(config: StorageFactory) -> Result<LocalStorage, (StorageError, StorageFactory)>
        where
            Self: Sized,
    {
        match serde_json::from_value::<LocalConfig>(config.handler_config.clone()) {
            Ok(local) => {
                let storage = LocalStorage {
                    storage_config: config.generic_config,
                    config: local,
                    repositories: RwLock::new(Default::default()),
                    status: StorageStatus::Unloaded,
                };
                Ok(storage)
            }
            Err(error) => Err((StorageError::JSONError(error), config)),
        }
    }
    async fn load(&mut self) -> Result<(), StorageError> {
        if self.status == StorageStatus::Loaded {
            return Err(StorageError::LoadFailure(
                "Attempted Double Load".to_string(),
            ));
        }
        let repositories =
            Self::load_repositories(PathBuf::from(&self.config.location).join(STORAGE_CONFIG))
                .await?;
        self.repositories = RwLock::new(repositories);
        return Ok(());
    }

    fn unload(&mut self) -> Result<(), StorageError> {
        let mut repositories = self.repositories.blocking_write();
        //repositories.drain() <--- Note to self. if we add a repository closing. Use this
        repositories.clear();
        self.status = StorageStatus::Unloaded;
        Ok(())
    }
    fn config_for_saving(&self) -> StorageSaver {
        let value = serde_json::to_value(self.config.clone()).unwrap();
        StorageSaver {
            storage_type: StorageType::LocalStorage,
            generic_config: self.storage_config.clone(),
            handler_config: value,
        }
    }

    fn storage_config(&self) -> &StorageConfig {
        &self.storage_config
    }

    fn impl_config(&self) -> Value {
        serde_json::to_value(self.config.clone()).unwrap()
    }

    fn storage_type(&self) -> &StorageType {
        &StorageType::LocalStorage
    }

    fn status(&self) -> &StorageStatus {
        &self.status
    }

    async fn create_repository(
        &self,
        name: String,
        repository_type: RepositoryType,
    ) -> Result<(), StorageError> {
        let mut repositories = self.repositories.write().await;
        let repository_folder = self.get_repository_folder(&name);
        if repositories.contains_key(&name) {
            return Err(StorageError::RepositoryAlreadyExists);
        }
        if repository_folder.exists() {
            warn!(
                "Creating {:?} on already existing files. This could result in unexpected behavior",
                &repository_folder
            );
        } else {
            create_dir_all(&repository_folder).await?;
        }
        let config = RepositoryConfig {
            name: name.clone(),
            repository_type,
            storage: self.storage_config.name.clone(),
            visibility: Default::default(),
            active: true,
            policy: Default::default(),
            created: get_current_time(),
        };
        let conf_folder = self
            .get_repository_folder(&name)
            .join(REPOSITORY_CONF_FOLDER);
        if !conf_folder.exists() {
            create_dir(&conf_folder).await?;
        }
        repositories.insert(name.clone(), config);
        drop(repositories);
        self.save_repositories().await?;
        Ok(())
    }

    async fn delete_repository(
        &self,
        repository: &RepositoryConfig,
        delete_files: bool,
    ) -> Result<(), StorageError> {
        let mut repositories = self.repositories.write().await;
        if repositories.remove(&repository.name).is_none() {
            return Err(RepositoryMissing);
        }
        let repository_folder = self.get_repository_folder(&repository.name);

        if delete_files {
            remove_dir(repository_folder).await?;
        } else {
            let conf_folder = repository_folder.join(REPOSITORY_CONF_FOLDER);
            if conf_folder.exists() {
                remove_dir(conf_folder).await?;
            }
        }
        drop(repositories);
        self.save_repositories().await?;
        Ok(())
    }

    async fn get_repositories(&self) -> Result<Vec<RepositoryConfig>, StorageError> {
        let mut repositories_res = Vec::new();

        let repositories = self.repositories.read().await;
        for (_, config) in repositories.iter() {
            repositories_res.push(config.clone());
        }
        return Ok(repositories_res);
    }

    async fn get_repository(
        &self,
        repository: &str,
    ) -> Result<Option<RwLockReadGuard<'_, RepositoryConfig>>, StorageError> {
        let repositories = self.repositories.read().await;
        if !repositories.contains_key(repository) {
            return Ok(None);
        }
        Ok(Some(RwLockReadGuard::map(repositories, |repos| {
            repos.get(repository).unwrap()
        })))
    }

    async fn update_repository(&self, repository: RepositoryConfig) -> Result<(), StorageError> {
        let mut repositories = self.repositories.write().await;
        if !repositories.contains_key(&repository.name) {
            return Err(RepositoryMissing);
        }
        repositories.insert(repository.name.clone(), repository);
        self.save_repositories().await?;
        return Ok(());
    }

    async fn update_repository_config(
        &self,
        repository: &RepositoryConfig,
        file: &str,
        data: &Option<Value>,
    ) -> Result<(), StorageError> {
        let repositories = self.repositories.write().await;
        if !repositories.contains_key(&repository.name) {
            return Err(RepositoryMissing);
        }
        let conf = self
            .get_repository_folder(&repository.name)
            .join(REPOSITORY_CONF_FOLDER)
            .join(file);

        if data.is_none() {
            remove_file(&conf).await?;
        } else if let Some(value) = data {
            let file = OpenOptions::new()
                .create(true)
                .open(&conf)
                .await?
                .into_std()
                .await;
            serde_json::to_writer_pretty(file, value)?;
        }
        return Ok(());
    }

    async fn get_repository_config(
        &self,
        repository: &RepositoryConfig,
        file: &str,
    ) -> Result<Option<Value>, StorageError> {
        let repositories = self.repositories.write().await;
        if !repositories.contains_key(&repository.name) {
            return Err(RepositoryMissing);
        }
        let conf = self
            .get_repository_folder(&repository.name)
            .join(REPOSITORY_CONF_FOLDER)
            .join(file);
        if !conf.exists() {
            return Ok(None);
        }
        let file = OpenOptions::new()
            .read(true)
            .open(&conf)
            .await?
            .into_std()
            .await;

        Ok(serde_json::from_reader(file)?)
    }

    async fn save_file(
        &self,
        repository: &RepositoryConfig,
        data: &[u8],
        location: &str,
    ) -> Result<bool, StorageError> {
        let file_location = self.get_repository_folder(&repository.name).join(location);
        trace!("Saving File {:?}", &file_location);
        create_dir_all(file_location.parent().ok_or(StorageError::ParentIssue)?).await?;

        let exists = if file_location.exists() {
            remove_file(&file_location).await?;
            true
        } else {
            false
        };
        let mut file = OpenOptions::new()
            .create_new(true)
            .open(&file_location)
            .await?;
        file.write_all(data).await?;

        Ok(exists)
    }

    async fn delete_file(
        &self,
        repository: &RepositoryConfig,
        location: &str,
    ) -> Result<(), StorageError> {
        let file_location = self.get_repository_folder(&repository.name).join(location);
        remove_file(file_location)
            .await
            .map_err(StorageError::IOError)
    }

    async fn get_file_as_response(
        &self,
        repository: &RepositoryConfig,
        location: &str,
    ) -> Result<StorageFileResponse, StorageError> {
        let file_location = self.get_repository_folder(&repository.name).join(location);
        if !file_location.exists() {
            return Ok(StorageFileResponse::NotFound);
        }
        if file_location.is_dir() {
            let mut path = format!("{}/{}", self.storage_config.name, repository.name);

            for x in location.split('/') {
                if !x.is_empty() {
                    path = format!("{}/{}", path, x);
                }
            }
            trace!("Directory Listing at {:?}", &path);
            let directory = StorageFile::create(&path, &file_location).await?;
            //Using STD because Into Iterator is missing
            let dir = std::fs::read_dir(&file_location)?;
            let mut files = Vec::new();
            for x in dir {
                let entry = x?;

                let name = entry.file_name().into_string();
                if name.is_err() {
                    continue;
                }
                let name = name.unwrap();
                if name.ends_with(".nitro_repo") || name.starts_with(".nitro_repo") {
                    //Hide All .nitro_repo files from File Listings
                    continue;
                }
                let relative_path = format!("{}/{}", path, &name);
                let result = StorageFile::create_from_entry(relative_path, &entry).await?;
                files.push(result);
            }
            let response = StorageDirectoryResponse { files, directory };
            return Ok(StorageFileResponse::List(response));
        }
        trace!("Returning File {:?}", &file_location);
        Ok(StorageFileResponse::File(file_location))
    }

    async fn get_file_information(
        &self,
        repository: &RepositoryConfig,
        location: &str,
    ) -> Result<Option<StorageFile>, StorageError> {
        let file_location = self.get_repository_folder(&repository.name).join(&location);
        if !file_location.exists() {
            return Ok(None);
        }

        return Ok(Some(StorageFile::create(location, &file_location).await?));
    }

    async fn get_file(
        &self,
        repository: &RepositoryConfig,
        location: &str,
    ) -> Result<Option<Vec<u8>>, StorageError> {
        let file_location = self.get_repository_folder(&repository.name).join(location);
        debug!("Storage File Request {}", file_location.to_str().unwrap());
        if !file_location.exists() {
            return Ok(None);
        }
        let mut file = OpenOptions::new().read(true).open(file_location).await?;
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes).await?;
        Ok(Some(bytes))
    }
}

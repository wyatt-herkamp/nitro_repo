use crate::repository::REPOSITORY_CONF;
use crate::storage::models::{StorageConfig, StorageFile, StorageFileResponse, StorageType};
use crate::storage::STORAGE_CONFIG;
use crate::utils::get_current_time;

use log::{debug, trace};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use std::io::{Read, Write};
use std::path::PathBuf;

use std::sync::Arc;

use crate::repository::data::{RepositoryConfig, RepositoryDataType, RepositoryMainConfig, RepositorySetting, RepositoryType, RepositoryValue};
use async_trait::async_trait;
use serde_json::Value;
use thiserror::Error;
use tokio::fs::{create_dir_all, read_to_string, remove_file, File, OpenOptions};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::RwLock;

#[derive(Error, Debug)]
pub enum LocalStorageError {
    #[error("Unable to Find Repository")]
    MissingRepository,
    #[error("Invalid Repository Type {0}")]
    InvalidRepositoryType(String),
    #[error("IO error {0}")]
    IOError(std::io::Error),
    #[error("JSON error {0}")]
    JSONError(serde_json::Error),
    #[error("{0}")]
    Error(String),
}

impl From<std::io::Error> for LocalStorageError {
    fn from(err: std::io::Error) -> LocalStorageError {
        LocalStorageError::IOError(err)
    }
}

impl From<serde_json::Error> for LocalStorageError {
    fn from(err: serde_json::Error) -> LocalStorageError {
        LocalStorageError::JSONError(err)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalConfig {
    pub location: String,
}

impl LocalStorage {
    pub async fn load(value: Value) -> Result<LocalStorage, LocalStorageError> {
        let config: LocalConfig = serde_json::from_value(value)?;
        Ok(LocalStorage {
            config,
            loaded: false,
            repositories: Arc::new(RwLock::new(Default::default())),
        })
    }
    async fn load_repositories(
        path: PathBuf,
    ) -> Result<HashMap<String, RepositoryValue>, LocalStorageError> {
        if !path.exists() {
            return Ok(HashMap::new());
        }
        let string = read_to_string(&path).await?;
        let result: Vec<RepositoryValue> = serde_json::from_str(&string)?;
        let mut values = HashMap::new();
        for x in result {
            values.insert(x.name.clone(), x);
        }
        Ok(values)
    }
}

#[derive(Debug, Clone)]
pub struct LocalStorage {
    pub config: LocalConfig,
    pub loaded: bool,
    pub repositories: Arc<RwLock<HashMap<String, RepositoryValue>>>,
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

#[async_trait]
impl StorageType for LocalStorage {
    type Error = LocalStorageError;
    type StorageConfig = LocalConfig;

    async fn create_repository(
        &self,
        _config: &StorageConfig,
        _name: String,
        _repository_type: RepositoryType,
    ) -> Result<RepositoryValue, Self::Error> {
        Err(LocalStorageError::Error("Not Implemented".to_string()))
    }

    async fn delete_repository<R: RepositoryDataType>(
        &self,
        _config: &StorageConfig,
        _repository: &R,
        _delete_files: bool,
    ) -> Result<(), Self::Error> {
        Err(LocalStorageError::Error("Not Implemented".to_string()))
    }

    async fn get_repositories(
        &self,
        _config: &StorageConfig,
    ) -> Result<Vec<RepositoryValue>, Self::Error> {
        let x = self.repositories.read().await;
        Ok(x.values().cloned().collect())
    }

    async fn get_repository<T: RepositorySetting>(
        &self,
        config: &StorageConfig,
        repo_name: &str,
    ) -> Result<Option<RepositoryConfig<T>>, Self::Error> {
        let value = self.get_repository_value(config, repo_name).await?;
        if value.is_none(){
            return Ok(None);
        }
        let path = self.get_repository_folder(repo_name).join(REPOSITORY_CONF);

        if !path.exists() {
            return Ok(None);
        }
        let string = read_to_string(&path).await?;
        let config: RepositoryMainConfig<Value> = serde_json::from_str(&string)?;
        let repository_type_settings =
            T::try_from(config.repository_type_settings).map_err(|_error| {
                Self::Error::Error(
                    "Error from T::try_from. I was really hoping this would never happen"
                        .to_string(),
                )
            })?;
        let result = RepositoryMainConfig::<T> {
            repository_type_settings,
            security: config.security,
            active: config.active,
            policy: config.policy,
        };

        Ok(Some(RepositoryConfig::<T>{
            init_values: value.unwrap(),
            main_config: result
        }))
    }

    async fn update_repository<RS: RepositorySetting>(
        &self,
        _config: &StorageConfig,
        _repository: RepositoryMainConfig<RS>,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    async fn save_file<R: RepositoryDataType>(
        &self,
        _config: &StorageConfig,
        repository: &R,
        data: &[u8],
        _location: &str,
    ) -> Result<(), Self::Error> {
        let file_location = self.get_repository_folder(repository.get_name());
        trace!("Saving File {:?}", &file_location);
        create_dir_all(file_location.parent().ok_or_else(|| {
            LocalStorageError::Error("Unable to Find Parent Location".to_string())
        })?)
            .await?;

        if file_location.exists() {
            remove_file(&file_location).await?;
        }
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .create(true)
            .open(&file_location)
            .await?;
        file.write_all(data).await?;
        Ok(())
    }

    async fn delete_file<R: RepositoryDataType>(
        &self,
        _config: &StorageConfig,
        repository: &R,
        location: &str,
    ) -> Result<(), Self::Error> {
        let file_location = self
            .get_repository_folder(repository.get_name())
            .join(location);
        remove_file(file_location).await?;
        Ok(())
    }

    async fn get_file_as_response<R: RepositoryDataType>(
        &self,
        config: &StorageConfig,
        repository: &R,
        location: &str,
    ) -> Result<Option<StorageFileResponse>, Self::Error> {
        let file_location = self
            .get_repository_folder(repository.get_name())
            .join(location);
        if !file_location.exists() {
            return Ok(None);
        }
        if file_location.is_dir() {
            let mut path = format!("{}/{}", config.name, repository.get_name());

            for x in location.split('/') {
                if !x.is_empty() {
                    path = format!("{}/{}", path, x);
                }
            }
            trace!("Directory Listing at {:?}", &path);
            //Using STD because Into Iterator is missing
            let dir = std::fs::read_dir(&file_location)?;
            let mut files = Vec::new();
            for x in dir {
                let entry = x?;
                let string = entry.file_name().into_string().unwrap();
                if string.ends_with(".nitro_repo") || string.starts_with(".nitro_repo") {
                    //Hide All .nitro_repo files from File Listings
                    continue;
                }
                let full = format!("{}/{}", path, &string);
                let metadata = entry.metadata().unwrap();
                let time = get_current_time() as u128;
                let file = StorageFile {
                    name: string,
                    full_path: full,
                    directory: entry.file_type()?.is_dir(),
                    file_size: metadata.len(),
                    created: time,
                };
                files.push(file);
            }

            return Ok(Some(StorageFileResponse::List(files)));
        }
        trace!("Returning File {:?}", &file_location);
        Ok(Some(StorageFileResponse::File(file_location)))
    }

    async fn get_file<R: RepositoryDataType>(
        &self,
        _config: &StorageConfig,
        repository: &R,
        location: &str,
    ) -> Result<Option<Vec<u8>>, Self::Error> {
        let file_location = self
            .get_repository_folder(repository.get_name())
            .join(location);

        debug!("Storage File Request {}", file_location.to_str().unwrap());
        if !file_location.exists() {
            return Ok(None);
        }
        let mut file = File::open(file_location).await?;
        let mut bytes = Vec::new();
        file.read(&mut bytes).await?;
        Ok(Some(bytes))
    }

    async fn load(&mut self, _config: &StorageConfig) -> Result<(), Self::Error> {
        if self.loaded {
            panic!("Attempted to Double Load Storage");
        }
        /// Load Repositories
        let repositories =
            Self::load_repositories(PathBuf::from(&self.config.location).join(STORAGE_CONFIG))
                .await?;
        self.repositories = Arc::new(RwLock::new(repositories));
        return Ok(());
    }

    fn unload(&mut self) -> Result<(), Self::Error> {
        let mut repositories = self.repositories.blocking_write();
        //repositories.drain() <--- Note to self. if we add a repository closing. Use this
        repositories.clear();
        self.loaded = false;
        Ok(())
    }

    async fn get_repository_value(&self, config: &StorageConfig, repository: &str) -> Result<Option<RepositoryValue>, Self::Error> {
        let repositories = self.repositories.read().await;
        return Ok(repositories.get(repository).cloned());
    }
}

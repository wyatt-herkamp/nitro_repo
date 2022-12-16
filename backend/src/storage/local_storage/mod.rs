use std::collections::HashMap;
use std::hint::unreachable_unchecked;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;

use crate::repository::handler::{DynamicRepositoryHandler, Repository};
use async_trait::async_trait;
use bytes::Bytes;
use futures::Stream;
use lockfree::map::{Map, Removed};
use log::{debug, error, trace, warn};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use tokio::fs::{
    create_dir_all, read_to_string, remove_dir_all, remove_file, OpenOptions,
};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use tokio_stream::StreamExt;

use crate::repository::settings::{RepositoryConfig, RepositoryConfigType};

use crate::storage::error::StorageError;

use crate::storage::file::{StorageDirectoryResponse, StorageFile, StorageFileResponse};

use crate::storage::models::{Storage, StorageStatus};
use crate::storage::path::{StoragePath, SystemStorageFile};
use crate::storage::{
    DynamicStorage, StorageConfig, StorageSaver, REPOSITORY_FOLDER, STORAGE_CONFIG,
};

#[derive(Debug)]
pub struct LocalStorage {
    pub config: LocalConfig,
    pub path: PathBuf,
    pub storage_config: StorageSaver,
    pub status: StorageStatus,
    pub repositories: Map<String, Arc<DynamicRepositoryHandler<DynamicStorage>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalConfig {
    pub location: String,
}

pub struct LocalFile {
    pub path: PathBuf,
}

impl LocalStorage {
    pub fn get_storage_folder(&self) -> &PathBuf {
        &self.path
    }
    pub fn get_repository_folder(&self, repository: &str) -> PathBuf {
        self.get_storage_folder().join(repository)
    }
}

impl LocalStorage {
    async fn load_repositories(
        path: PathBuf,
    ) -> Result<HashMap<String, RepositoryConfig>, StorageError> {
        trace!("Loading repositories from {}", path.display());
        if !path.exists() {
            create_dir_all(&path).await?;
            warn!("Repositories folder does not exist");
            return Ok(HashMap::new());
        }
        let storage_conf = path.join(STORAGE_CONFIG);
        if !storage_conf.exists() {
            warn!("Repositories file exist {storage_conf:?}");
            return Ok(HashMap::new());
        }
        let string = read_to_string(storage_conf).await?;
        let result: Vec<String> = serde_json::from_str(&string)?;
        let mut values = HashMap::new();
        for x in result {
            let buf = path
                .join(x.as_str())
                .join(REPOSITORY_FOLDER)
                .join(RepositoryConfig::config_name());
            let result: Result<RepositoryConfig, _> =
                serde_json::from_reader(std::fs::OpenOptions::new().read(true).open(buf)?);
            match result {
                Ok(result) => {
                    values.insert(x, result);
                }
                Err(error) => {
                    error!("Unable to load {x}. Error {error:?}")
                }
            }
        }
        Ok(values)
    }
    async fn save_repositories(&self) -> Result<(), StorageError> {
        let conf = self.get_storage_folder().join(STORAGE_CONFIG);

        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .open(conf)
            .await?;
        let values: Vec<String> = self.repositories.iter().map(|v| v.key().clone()).collect();
        let string = serde_json::to_string_pretty(&values)?;
        file.write_all(string.as_bytes()).await?;
        Ok(())
    }
}
macro_rules! unsafe_into_config {
    ($value:ident) => {
        if let StorageConfig::LocalStorage(local) = &$value.handler_config {
            local
        } else {
            unsafe { unreachable_unchecked() }
        }
    };
}
#[async_trait]
impl Storage for LocalStorage {
    type Repository = DynamicRepositoryHandler<DynamicStorage>;

    async fn create_new(mut config: StorageSaver) -> Result<Self, (StorageError, StorageSaver)>
    where
        Self: Sized,
    {
        let mut local_config = unsafe_into_config!(config).clone();
        if local_config.location.contains("{local_storage_folder}") {
            unsafe {
                let result = std::env::var("STORAGE_LOCATION").unwrap_unchecked();
                local_config.location = local_config
                    .location
                    .replace("{local_storage_folder}", &result);
            }
        }
        if local_config.location.contains("{storage_id}") {
            local_config.location = local_config
                .location
                .replace("{storage_id}", &config.generic_config.id);
        }
        config.handler_config = StorageConfig::LocalStorage(local_config.clone());
        let buf = PathBuf::from_str(local_config.location.as_ref()).unwrap();

        let storage = LocalStorage {
            config: local_config,
            path: buf,
            storage_config: config,
            status: StorageStatus::Loaded,
            repositories: Map::new(),
        };
        Ok(storage)
    }

    async fn new(config: StorageSaver) -> Result<Self, (StorageError, StorageSaver)>
    where
        Self: Sized,
    {
        let v = unsafe_into_config!(config).clone();
        let buf = PathBuf::from_str(v.location.as_ref()).unwrap();
        let storage = LocalStorage {
            config: v,
            path: buf,
            storage_config: config,
            status: StorageStatus::Unloaded,
            repositories: Map::new(),
        };
        Ok(storage)
    }

    async fn get_repos_to_load(&self) -> Result<HashMap<String, RepositoryConfig>, StorageError> {
        let repositories = Self::load_repositories(PathBuf::from(&self.config.location)).await?;
        Ok(repositories)
    }

    fn add_repo_loaded<R: Into<Self::Repository> + Send>(
        &self,
        repo: R,
    ) -> Result<(), StorageError> {
        let repo = repo.into();
        self.repositories
            .insert(repo.get_repository().name.clone(), Arc::new(repo));

        Ok(())
    }

    fn unload(&mut self) -> Result<(), StorageError> {
        //repositories.drain() <--- Note to self. if we add a repository closing. Use this
        self.repositories.clear();
        self.status = StorageStatus::Unloaded;
        Ok(())
    }

    fn storage_config(&self) -> &StorageSaver {
        &self.storage_config
    }

    async fn create_repository<R: Into<Self::Repository> + Send>(
        &self,
        repository: R,
    ) -> Result<Arc<Self::Repository>, StorageError> {
        let repository = repository.into();
        if self
            .repositories
            .get(repository.get_repository().name.as_str())
            .is_some()
        {
            return Err(StorageError::RepositoryAlreadyExists);
        }
        let path = self.get_repository_folder(&repository.get_repository().name);
        let config_dir = path.join(REPOSITORY_FOLDER);
        if !config_dir.exists() {
            create_dir_all(&config_dir).await?;
        }
        let mut buf = OpenOptions::new()
            .create(true)
            .write(true)
            .open(config_dir.join(RepositoryConfig::config_name()))
            .await?;
        let result = serde_json::to_string_pretty(repository.get_repository())?;
        buf.write_all(result.as_bytes()).await?;
        drop(buf);
        let name = repository.get_repository().name.clone();
        self.repositories.insert(name.clone(), Arc::new(repository));
        self.save_repositories().await?;
        Ok(self.repositories.get(name.as_str()).unwrap().val().clone())
    }

    async fn delete_repository<S: AsRef<str> + Send>(
        &self,
        repository: S,
        delete_files: bool,
    ) -> Result<(), StorageError> {
        let _ = self
            .repositories
            .remove(repository.as_ref())
            .ok_or(StorageError::RepositoryMissing)?;
        self.save_repositories().await?;

        if delete_files {
            let path = self.get_repository_folder(repository.as_ref());
            remove_dir_all(path).await?;
        } else {
            let path = self.get_repository_folder(repository.as_ref());
            remove_dir_all(path.join(".config.nitro_repo")).await?;
        }
        Ok(())
    }

    fn get_repository_list(&self) -> Result<Vec<RepositoryConfig>, StorageError> {
        Ok(self
            .repositories
            .iter()
            .map(|v| v.val().get_repository().clone())
            .collect())
    }

    fn get_repository<S: AsRef<str>>(
        &self,
        repository: S,
    ) -> Result<Option<Arc<Self::Repository>>, StorageError> {
        let option = self
            .repositories
            .get(repository.as_ref())
            .map(|v| v.val().clone());
        Ok(option)
    }

    fn remove_repository_for_updating<S: AsRef<str>>(
        &self,
        repository: S,
    ) -> Option<Removed<String, Arc<Self::Repository>>> {
        self.repositories.remove(repository.as_ref())
    }

    async fn add_repository_for_updating(
        &self,
        name: String,
        repository_arc: Self::Repository,
        save: bool,
    ) -> Result<(), StorageError> {
        self.repositories.insert(name, Arc::new(repository_arc));
        if save {
            self.save_repositories().await?;
        }
        Ok(())
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
            .write(true)
            .open(&file_location)
            .await?;
        file.write_all(data).await?;

        Ok(exists)
    }

    fn write_file_stream<S: Stream<Item = Bytes> + Unpin + Send + Sync + 'static>(
        &self,
        repository: &RepositoryConfig,
        s: S,
        location: &str,
    ) -> Result<bool, StorageError> {
        let file_location = self.get_repository_folder(&repository.name).join(location);
        trace!("Saving File {:?}", &file_location);
        std::fs::create_dir_all(file_location.parent().ok_or(StorageError::ParentIssue)?)?;

        let exists = file_location.exists();
        let existss = exists;
        tokio::spawn(async move {
            let mut s = s;
            if existss {
                remove_file(&file_location)
                    .await
                    .expect("Failed to remove file");
            }
            let mut file_location = OpenOptions::new()
                .create_new(true)
                .write(true)
                .open(file_location)
                .await
                .expect("Failed to open file");
            while let Some(chunk) = s.next().await {
                file_location
                    .write_all(chunk.as_ref())
                    .await
                    .expect("Failed to write file");
            }
            trace!("File saved");
        });
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
        trace!("Getting File {}", &file_location.display());
        if !file_location.exists() {
            return Ok(StorageFileResponse::NotFound);
        }
        if file_location.is_dir() {
            let mut path = format!(
                "{}/{}",
                self.storage_config.generic_config.id, repository.name
            );

            for x in location.split('/') {
                if !x.is_empty() {
                    path = format!("{}/{}", path, x);
                }
            }
            trace!("Directory Listing at {:?}", &path);
            let directory = StorageFile::create(&path, &file_location).await?;
            //Using STD because Into Iterator is missing
            trace!("Meta Data {:?}", &directory);
            let dir = std::fs::read_dir(&file_location)?;
            trace!("Dir Read {:?}", &dir);
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

    async fn get_repository_config<ConfigType: DeserializeOwned>(
        &self,
        repository: &RepositoryConfig,
        config_name: &str,
    ) -> Result<Option<ConfigType>, StorageError> {
        let buf = self
            .get_repository_folder(repository.name.as_str())
            .join(".config.nitro_repo")
            .join(config_name);
        if !buf.exists() {
            Ok(None)
        } else {
            let string = read_to_string(buf).await.map_err(StorageError::IOError)?;
            serde_json::from_str(&string)
                .map_err(StorageError::JSONError)
                .map(Some)
        }
    }

    async fn save_repository_config<ConfigType: RepositoryConfigType>(
        &self,
        repository: &RepositoryConfig,
        config: &ConfigType,
    ) -> Result<(), StorageError> {
        let buf = self
            .get_repository_folder(repository.name.as_str())
            .join(".config.nitro_repo")
            .join(ConfigType::config_name());
        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .open(buf)
            .await?;
        let string = serde_json::to_string_pretty(config).map_err(StorageError::JSONError)?;
        file.write_all(string.as_bytes()).await?;
        Ok(())
    }

    async fn list_files<S: AsRef<str> + Send, SP: Into<StoragePath> + Send>(
        &self,
        repository: S,
        path: SP,
    ) -> Result<Vec<SystemStorageFile>, StorageError> {
        let path = path.into();
        let system_path = path.join_system(self.get_repository_folder(repository.as_ref()));
        let dir = std::fs::read_dir(&system_path)?;
        let mut files = Vec::new();
        for value in dir {
            let entry = value?;

            files.push(SystemStorageFile {
                name: entry
                    .file_name()
                    .into_string()
                    .expect("Failed to get file name"),
                is_dir: entry.path().is_dir(),
            });
        }
        Ok(files)
    }
}

use crate::repository::models::{Repository, RepositorySummary};
use crate::repository::types::RepositoryType;
use crate::repository::{REPOSITORY_CONF, REPOSITORY_CONF_BAK};
use crate::storage::models::{
    FileResponse, RepositoriesFile, StorageConfig, StorageFile, StorageType,
};
use crate::storage::{StorageFileResponse, STORAGE_CONFIG};
use crate::utils::get_current_time;
use actix_files::NamedFile;
use actix_web::HttpRequest;
use either::Either;
use log::{debug, info, trace, warn};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use std::io::{Read, Write};
use std::path::PathBuf;
use std::str::FromStr;

use crate::api_response::SiteResponse;

use async_trait::async_trait;
use thiserror::Error;
use tokio::fs::{
    create_dir_all, read_to_string, remove_dir_all, remove_file, rename, File, OpenOptions,
};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

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
pub struct LocalStorage {
    pub location: String,
}

pub struct LocalFile {
    pub path: PathBuf,
}

impl StorageFileResponse for LocalFile {
    fn to_request(self, request: &HttpRequest) -> SiteResponse {
        Ok(NamedFile::open(self.path)?.into_response(request))
    }
}

impl LocalStorage {
    pub fn get_storage_folder(&self) -> PathBuf {
        PathBuf::from(&self.location)
    }
    pub fn get_repository_folder(&self, repository: &str) -> PathBuf {
        self.get_storage_folder().join(repository)
    }
}

#[async_trait]
impl StorageType<LocalFile> for LocalStorage {
    type Error = LocalStorageError;

    async fn init(&self, _config: &StorageConfig) -> Result<(), Self::Error> {
        let path = self.get_storage_folder();
        if !path.exists() {
            create_dir_all(&path).await?;
        }
        let buf = path.join(STORAGE_CONFIG);
        if buf.exists() {
            remove_file(&buf).await?
        }

        Ok(())
    }

    async fn create_repository(
        &self,
        _config: &StorageConfig,
        repository: RepositorySummary,
    ) -> Result<Repository, Self::Error> {
        let storages = self.get_storage_folder();
        let location = storages.join(&repository.name);

        {
            let storage_config = storages.join(STORAGE_CONFIG);
            info!(
                "Adding Value to Storage Config {}",
                storage_config.to_str().unwrap()
            );
            let mut repos = if storage_config.exists() {
                let string = read_to_string(&storage_config).await?;
                remove_file(&storage_config).await?;
                serde_json::from_str(&string)?
            } else {
                HashMap::<String, RepositorySummary>::new()
            };
            repos.insert(repository.name.clone(), repository.clone());
            let result = serde_json::to_string_pretty(&repos)?;

            let mut file = OpenOptions::new()
                .write(true)
                .create(true)
                .open(storage_config)
                .await?;
            file.write_all(result.as_bytes()).await?;
        }
        info!("Creating Directory {}", location.to_str().unwrap());
        let typ: RepositoryType = RepositoryType::from_str(&repository.repo_type)
            .map_err(|typ| LocalStorageError::InvalidRepositoryType(typ.to_string()))?;
        create_dir_all(&location).await?;
        let repo = Repository {
            name: repository.name,
            repo_type: typ,
            storage: repository.storage,
            settings: Default::default(),
            security: Default::default(),
            deploy_settings: Default::default(),
            created: get_current_time(),
        };
        let result = serde_json::to_string_pretty(&repo)?;

        let config = location.join(REPOSITORY_CONF);
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .open(config)
            .await?;
        file.write_all(result.as_bytes()).await?;
        Ok(repo)
    }

    async fn delete_repository(
        &self,
        _config: &StorageConfig,
        repository: &Repository,
        delete_files: bool,
    ) -> Result<(), Self::Error> {
        let storage_location = self.get_storage_folder();

        let storage_config = storage_location.join(STORAGE_CONFIG);
        let mut repos = if storage_config.exists() {
            let string = read_to_string(&storage_config).await?;
            remove_file(&storage_config).await?;
            serde_json::from_str(&string)?
        } else {
            HashMap::<String, RepositorySummary>::new()
        };
        repos.remove(&repository.name);
        let result = serde_json::to_string_pretty(&repos)?;

        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .open(storage_config)
            .await?;
        file.write_all(result.as_bytes()).await?;
        let path = storage_location.join(&repository.name);

        if delete_files {
            warn!(
                "All Files for {} are being deleted",
                &path.to_str().unwrap()
            );
            remove_dir_all(&path).await?;
        } else {
            let config = storage_location
                .join(&repository.name)
                .join(REPOSITORY_CONF);
            remove_file(config).await?;
        }
        Ok(())
    }

    async fn get_repositories(
        &self,
        _config: &StorageConfig,
    ) -> Result<RepositoriesFile, Self::Error> {
        let path = self.get_storage_folder().join(STORAGE_CONFIG);
        if !path.exists() {
            return Ok(HashMap::new());
        }
        let string = read_to_string(&path).await?;
        let result: RepositoriesFile = serde_json::from_str(&string)?;
        Ok(result)
    }

    async fn get_repository(
        &self,
        _config: &StorageConfig,
        repo_name: &str,
    ) -> Result<Option<Repository>, Self::Error> {
        let path = self.get_repository_folder(repo_name).join(REPOSITORY_CONF);

        if !path.exists() {
            return Ok(None);
        }
        let string = read_to_string(&path).await?;
        let result: Repository = serde_json::from_str(&string)?;
        Ok(Some(result))
    }

    async fn update_repository(
        &self,
        _config: &StorageConfig,
        repository: &Repository,
    ) -> Result<(), Self::Error> {
        let location = self.get_repository_folder(&repository.name);
        let config = location.join(REPOSITORY_CONF);
        let bak = location.join(REPOSITORY_CONF_BAK);
        if !config.exists() {
            return Err(LocalStorageError::MissingRepository);
        }
        if bak.exists() {
            remove_file(&bak).await?;
        }
        rename(&config, bak).await?;
        let result = serde_json::to_string(repository)?;
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .open(config)
            .await?;
        file.write_all(result.as_bytes()).await?;
        Ok(())
    }

    async fn save_file(
        &self,
        _config: &StorageConfig,
        repository: &Repository,
        data: &[u8],
        _location: &str,
    ) -> Result<(), Self::Error> {
        let file_location = self.get_repository_folder(&repository.name);
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

    async fn delete_file(
        &self,
        _config: &StorageConfig,
        repository: &Repository,
        location: &str,
    ) -> Result<(), Self::Error> {
        let file_location = self.get_repository_folder(&repository.name).join(location);
        remove_file(file_location).await?;
        Ok(())
    }

    async fn get_file_as_response(
        &self,
        config: &StorageConfig,
        repository: &Repository,
        location: &str,
    ) -> Result<Option<FileResponse<LocalFile>>, Self::Error> {
        let file_location = self.get_repository_folder(&repository.name).join(location);
        if !file_location.exists() {
            return Ok(None);
        }
        if file_location.is_dir() {
            let mut path = format!("{}/{}", config.name, repository.name);

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

            return Ok(Some(Either::Right(files)));
        }
        trace!("Returning File {:?}", &file_location);
        Ok(Some(Either::Left(LocalFile {
            path: file_location,
        })))
    }

    async fn get_file(
        &self,
        _config: &StorageConfig,
        repository: &Repository,
        location: &str,
    ) -> Result<Option<Vec<u8>>, Self::Error> {
        let file_location = self.get_repository_folder(&repository.name).join(location);

        debug!("Storage File Request {}", file_location.to_str().unwrap());
        if !file_location.exists() {
            return Ok(None);
        }
        let mut file = File::open(file_location).await?;
        let mut bytes = Vec::new();
        file.read(&mut bytes).await?;
        Ok(Some(bytes))
    }
}

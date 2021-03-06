use crate::error::internal_error::InternalError;
use crate::repository::models::{Repository, RepositorySummary};
use crate::repository::{REPOSITORY_CONF, REPOSITORY_CONF_BAK};
use crate::storage::models::{LocationType, Storage};
use crate::storage::{
    FileResponse, LocationHandler, RepositoriesFile, StorageFile, StorageFileResponse,
    STORAGE_CONFIG,
};
use crate::utils::get_current_time;
use crate::{SiteResponse, StringMap};
use actix_files::NamedFile;
use actix_web::HttpRequest;
use either::Either;
use log::{debug, info, trace, warn};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::fs::{
    create_dir_all, read_dir, read_to_string, remove_dir_all, remove_file, File, OpenOptions,
};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;
use crate::repository::types::RepositoryType;
use std::str::FromStr;

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
    pub fn get_storage_folder(storage: &Storage<StringMap>) -> PathBuf {
        let location = storage.location.get("location").unwrap();
        PathBuf::from(location.clone())
    }
    pub fn get_repository_folder(storage: &Storage<StringMap>, repository: &str) -> PathBuf {
        LocalStorage::get_storage_folder(storage).join(repository)
    }
}

impl LocationHandler<LocalFile> for LocalStorage {
    fn init(storage: &Storage<StringMap>) -> Result<(), InternalError> {
        let location = storage.location.get("location").unwrap();
        let path = Path::new(location);
        if !path.exists() {
            create_dir_all(&path)?;
        }
        let buf = path.join(STORAGE_CONFIG);
        if buf.exists() {
            remove_file(&buf)?
        }

        Ok(())
    }

    fn create_repository(
        storage: &Storage<StringMap>,
        repository: RepositorySummary,
    ) -> Result<Repository, InternalError> {
        let location = storage.location.get("location").unwrap();
        let storages = Path::new(location);
        let location = storages.join(&repository.name);

        {
            let storage_config = storages.join(STORAGE_CONFIG);
            info!(
                "Adding Value to Storage Config {}",
                storage_config.to_str().unwrap()
            );
            let mut repos = if storage_config.exists() {
                let string = read_to_string(&storage_config)?;
                remove_file(&storage_config)?;
                serde_json::from_str(&string)?
            } else {
                HashMap::<String, RepositorySummary>::new()
            };
            repos.insert(repository.name.clone(), repository.clone());
            let result = serde_json::to_string_pretty(&repos)?;

            let mut file = OpenOptions::new()
                .write(true)
                .create(true)
                .open(storage_config)?;
            file.write_all(result.as_bytes())?;
        }
        info!("Creating Directory {}", location.to_str().unwrap());
        let typ: RepositoryType = RepositoryType::from_str(&repository.repo_type).map_err(|typ| {
            InternalError::InvalidRepositoryType(typ.to_string())
        })?;
        create_dir_all(&location)?;
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
        let mut file = OpenOptions::new().write(true).create(true).open(config)?;
        file.write_all(result.as_bytes())?;
        Ok(repo)
    }

    fn delete_repository(
        storage: &Storage<StringMap>,
        repository: &Repository,
        delete_files: bool,
    ) -> Result<(), InternalError> {
        let location = storage.location.get("location").unwrap();
        let storage_location = Path::new(location);

        let storage_config = storage_location.join(STORAGE_CONFIG);
        let mut repos = if storage_config.exists() {
            let string = read_to_string(&storage_config)?;
            remove_file(&storage_config)?;
            serde_json::from_str(&string)?
        } else {
            HashMap::<String, RepositorySummary>::new()
        };
        repos.remove(&repository.name);
        let result = serde_json::to_string_pretty(&repos)?;

        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .open(storage_config)?;
        file.write_all(result.as_bytes())?;
        let path = storage_location.join(&repository.name);

        if delete_files {
            warn!(
                "All Files for {} are being deleted",
                &path.to_str().unwrap()
            );
            remove_dir_all(&path)?;
        } else {
            let config = storage_location
                .join(&repository.name)
                .join(REPOSITORY_CONF);
            remove_file(config)?;
        }
        Ok(())
    }

    fn get_repositories(storage: &Storage<StringMap>) -> Result<RepositoriesFile, InternalError> {
        let location = storage.location.get("location").unwrap();

        let path = Path::new(location).join(STORAGE_CONFIG);
        if !path.exists() {
            return Ok(HashMap::new());
        }
        let string = read_to_string(&path)?;
        let result: RepositoriesFile = serde_json::from_str(&string)?;
        Ok(result)
    }

    fn get_repository(
        storage: &Storage<StringMap>,
        repository: &str,
    ) -> Result<Option<Repository>, InternalError> {
        let location = storage.location.get("location").unwrap();
        let path = Path::new(location).join(repository).join(REPOSITORY_CONF);

        if !path.exists() {
            return Ok(None);
        }
        let string = read_to_string(&path)?;
        let result: Repository = serde_json::from_str(&string)?;
        Ok(Some(result))
    }

    fn update_repository(
        storage: &Storage<StringMap>,
        repository: &Repository,
    ) -> Result<(), InternalError> {
        let location = storage.location.get("location").unwrap();
        let location = Path::new(location).join(&repository.name);
        let config = location.join(REPOSITORY_CONF);
        let bak = location.join(REPOSITORY_CONF_BAK);
        if !config.exists() {
            return Err(InternalError::NotFound);
        }
        if bak.exists() {
            fs::remove_file(&bak)?;
        }
        fs::rename(&config, bak)?;
        let result = serde_json::to_string(repository)?;
        let mut file = OpenOptions::new().write(true).create(true).open(config)?;
        file.write_all(result.as_bytes())?;
        Ok(())
    }

    fn save_file(
        storage: &Storage<StringMap>,
        repository: &Repository,
        data: &[u8],
        location: &str,
    ) -> Result<(), InternalError> {
        let file_location =
            LocalStorage::get_repository_folder(storage, &repository.name).join(location);
        trace!("Saving File {:?}", &file_location);
        create_dir_all(
            file_location
                .parent()
                .ok_or_else(|| InternalError::from("Unable to Find Parent Location"))?,
        )?;

        if file_location.exists() {
            remove_file(&file_location)?;
        }
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .create(true)
            .open(&file_location)?;
        file.write_all(data)?;
        Ok(())
    }

    fn delete_file(
        storage: &Storage<StringMap>,
        repository: &Repository,
        location: &str,
    ) -> Result<(), InternalError> {
        let file_location =
            LocalStorage::get_repository_folder(storage, &repository.name).join(location);
        remove_file(file_location)?;
        Ok(())
    }

    fn get_file_as_response(
        storage: &Storage<StringMap>,
        repository: &Repository,
        location: &str,
    ) -> Result<FileResponse<LocalFile>, InternalError> {
        let file_location =
            LocalStorage::get_repository_folder(storage, &repository.name).join(location);
        if !file_location.exists() {
            return Ok(Either::Right(vec![]));
        }
        if file_location.is_dir() {
            let mut path = format!("{}/{}", storage.name, repository.name);

            for x in location.split('/') {
                if !x.is_empty() {
                    path = format!("{}/{}", path, x);
                }
            }
            trace!("Directory Listing at {:?}", &path);

            let dir = read_dir(&file_location)?;
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
                let time = metadata
                    .created()
                    .unwrap()
                    .duration_since(UNIX_EPOCH)?
                    .as_millis();
                let file = StorageFile {
                    name: string,
                    full_path: full,
                    directory: entry.file_type()?.is_dir(),
                    file_size: metadata.len(),
                    created: time,
                };
                files.push(file);
            }

            return Ok(Either::Right(files));
        }
        trace!("Returning File {:?}", &file_location);
        Ok(Either::Left(LocalFile {
            path: file_location,
        }))
    }
    fn get_file(
        storage: &Storage<StringMap>,
        repository: &Repository,
        location: &str,
    ) -> Result<Option<Vec<u8>>, InternalError> {
        let file_location =
            LocalStorage::get_repository_folder(storage, &repository.name).join(location);

        debug!("Storage File Request {}", file_location.to_str().unwrap());
        if !file_location.exists() {
            return Ok(None);
        }
        let mut file = File::open(file_location)?;
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes)?;
        Ok(Some(bytes))
    }
}

impl TryFrom<Storage<StringMap>> for Storage<LocalStorage> {
    type Error = InternalError;

    fn try_from(value: Storage<StringMap>) -> Result<Self, Self::Error> {
        Ok(Self {
            public_name: value.public_name,
            name: value.name,
            created: value.created,
            location_type: LocationType::LocalStorage,
            location: LocalStorage::try_from(value.location)?,
        })
    }
}

impl TryFrom<StringMap> for LocalStorage {
    type Error = InternalError;

    fn try_from(mut value: StringMap) -> Result<LocalStorage, Self::Error> {
        let location = value
            .remove("location")
            .ok_or_else(|| InternalError::ConfigError("storage missing location".to_string()))?;

        Ok(LocalStorage { location })
    }
}

use std::{borrow::Cow, str::FromStr};

use ahash::HashMap;
use axum::response::{IntoResponse, Response};
use http::{HeaderValue, header::ToStrError};
use nr_core::{
    database::entities::project::{NewProject, versions::NewVersion},
    repository::project::VersionData,
    storage::{StoragePath, StoragePathComponent},
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use strum::{Display, EnumString};
use tracing::{debug, info};
use uuid::Uuid;

use crate::repository::{maven::get_release_type, npm::NPMRegistryError};

use super::NPMPackageName;

#[derive(Debug, Display, EnumString)]
pub enum NPMCommand {
    #[strum(serialize = "publish")]
    Publish,
}
impl TryFrom<&HeaderValue> for NPMCommand {
    type Error = InvalidNPMCommand;
    fn try_from(value: &HeaderValue) -> Result<Self, Self::Error> {
        let value = value.to_str()?;
        NPMCommand::from_str(value)
            .map_err(|_| InvalidNPMCommand::InvalidCommand(value.to_string()))
    }
}
#[derive(Debug, thiserror::Error)]
pub enum InvalidNPMCommand {
    #[error("Invalid command {0}")]
    InvalidCommand(String),
    #[error("Unparsable command {0}")]
    UnparsableCommand(#[from] ToStrError),
    #[error("No header found")]
    NoHeaderFound,
}
impl IntoResponse for InvalidNPMCommand {
    fn into_response(self) -> Response {
        Response::builder()
            .status(http::StatusCode::BAD_REQUEST)
            .body(self.to_string().into())
            .unwrap()
    }
}
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct PublishVersion {
    pub name: NPMPackageName,
    pub version: String,
    pub dist: PublishDist,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
    #[serde(rename = "_id")]
    pub hidden_id: String,
    #[serde(default)]
    pub readme: String,
    #[serde(default, rename = "readmeFilename")]
    pub readme_file_name: String,
    #[serde(rename = "_nodeVersion")]
    pub secret_node_version: String,
    #[serde(rename = "_npmVersion")]
    pub hidden_npm_version: String,
}
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct PublishDist {
    pub integrity: String,
    pub shasum: String,
    pub tarball: String,
    #[serde(flatten)]
    pub other: HashMap<String, Value>,
}
impl PublishDist {
    #[tracing::instrument]
    pub fn validate_tarball(
        &self,
        storage_name: &str,
        repository_name: &str,
    ) -> Result<(), NPMRegistryError> {
        let url = url::Url::from_str(&self.tarball).map_err(|error| {
            info!(?error, "Invalid tarball");
            NPMRegistryError::InvalidTarball {
                tarball_route: self.tarball.clone(),
                error: Cow::Owned(format!("Invalid URL: {}", error)),
            }
        })?;
        let mut path = url
            .path_segments()
            .ok_or(NPMRegistryError::InvalidTarball {
                tarball_route: self.tarball.clone(),
                error: Cow::Borrowed("No Path"),
            })?;
        if path.next().is_none() {
            info!(?url, "Invalid tarball (Missing Base Path for tarball)");
            return Err(NPMRegistryError::InvalidTarball {
                tarball_route: self.tarball.clone(),
                error: Cow::Borrowed("Missing base path"),
            });
        }
        if path.next() != Some(storage_name) {
            info!(?url, "Invalid tarball (Missing storage name)");
            return Err(NPMRegistryError::InvalidTarball {
                tarball_route: self.tarball.clone(),
                error: Cow::Borrowed("Missing storage name"),
            });
        }
        if path.next() != Some(repository_name) {
            info!(?url, "Invalid tarball (Missing repository name)");
            return Err(NPMRegistryError::InvalidTarball {
                tarball_route: self.tarball.clone(),
                error: Cow::Borrowed("Missing repository name"),
            });
        }
        Ok(())
    }
}
impl PublishVersion {
    pub fn new_project(
        &self,
        save_path: String,
        repository_id: Uuid,
    ) -> Result<NewProject, NPMRegistryError> {
        let project_key = self.name.to_string();
        let NPMPackageName { name, scope } = self.name.clone();
        Ok(NewProject {
            scope,
            project_key,
            name,
            storage_path: save_path,
            repository: repository_id,
            latest_pre_release: None,
            latest_release: None,
            description: None,
            tags: vec![],
        })
    }
    pub fn new_version(
        &self,
        project_id: Uuid,
        save_path: String,
        publisher: i32,
    ) -> Result<NewVersion, NPMRegistryError> {
        let release_type = get_release_type(&self.version);
        let extra = VersionData {
            extra: Some(serde_json::to_value(self).unwrap()),
            ..Default::default()
        };
        Ok(NewVersion {
            project_id,
            version: self.version.clone(),
            release_type,
            version_path: save_path,
            publisher: Some(publisher),
            version_page: None,
            extra,
        })
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GetPath {
    RegistryBase,
    Search,
    GetPackageInfo {
        name: String,
    },
    VersionInfo {
        name: String,
        version: String,
    },
    GetTar {
        name: String,
        version: String,
        file: String,
    },
}
impl GetPath {
    /// Path Types
    ///
    /// - `@{scope}/{package}` - Get package info
    /// - `@{scope}/{package}/{version}` - Get version info
    /// - `@{scope}/{package}/-/{scope}/{package}-{version}.tgz` - Get file
    pub fn scoped_package_call(
        components: Vec<StoragePathComponent>,
    ) -> Result<Self, NPMRegistryError> {
        let length = components.len();
        if length == 1 {
            panic!("Invalid path");
        }
        let name = format!("{}/{}", components[0], components[1]);
        if length == 2 {
            return Ok(GetPath::GetPackageInfo { name });
        }
        if length == 3 {
            let version = components[2].to_string();
            debug!(?name, ?version, "Version info");
            return Ok(GetPath::VersionInfo { name, version });
        }
        if length == 5 {
            let file = components[4].to_string();
            let version =
                extract_version_from_file(&file).ok_or(NPMRegistryError::InvalidGetRequest)?;
            return Ok(GetPath::GetTar {
                name,
                version,
                file,
            });
        }
        info!(?components, "Invalid path");
        Err(NPMRegistryError::InvalidGetRequest)
    }
    /// Path Types
    ///
    /// - `{package}` - Get package info
    /// - `{package}/{version}` - Get version info
    /// - `{package}/-/{package}-{version}.tgz` - Get file
    pub fn unscoped_package_call(
        components: Vec<StoragePathComponent>,
    ) -> Result<Self, NPMRegistryError> {
        let length = components.len();

        let name = components[0].to_string();
        if length == 1 {
            return Ok(GetPath::GetPackageInfo { name });
        }
        if length == 2 {
            let version = components[1].to_string();
            debug!(?name, ?version, "Version info");
            return Ok(GetPath::VersionInfo { name, version });
        }
        if length == 3 {
            let file = components[2].to_string();
            let version =
                extract_version_from_file(&file).ok_or(NPMRegistryError::InvalidGetRequest)?;
            return Ok(GetPath::GetTar {
                name,
                version,
                file,
            });
        }
        info!(?components, "Invalid path");
        Err(NPMRegistryError::InvalidGetRequest)
    }
}
impl TryFrom<StoragePath> for GetPath {
    type Error = NPMRegistryError;

    fn try_from(value: StoragePath) -> Result<Self, Self::Error> {
        let as_string = value.to_string();
        let components: Vec<_> = value.into();
        if as_string.starts_with('@') {
            GetPath::scoped_package_call(components)
        } else {
            GetPath::unscoped_package_call(components)
        }
    }
}
pub fn extract_version_from_file(file: &str) -> Option<String> {
    let parts: Vec<_> = file.split('-').collect();
    if let Some(version) = parts.last() {
        let version = version.trim_end_matches(".tgz");
        return Some(version.to_string());
    }
    None
}

#[cfg(test)]
pub mod tests {
    use nr_core::storage::StoragePath;

    use super::GetPath;
    #[test]
    pub fn tests() {
        let tests = vec![
            (
                StoragePath::from("@nr/mylib/-/@nr/mylib-1.0.0.tgz"),
                GetPath::GetTar {
                    name: "@nr/mylib".to_string(),
                    version: "1.0.0".to_string(),
                    file: "mylib-1.0.0.tgz".to_string(),
                },
            ),
            (
                StoragePath::from("mylib/-/mylib-1.0.0.tgz"),
                GetPath::GetTar {
                    name: "mylib".to_string(),
                    version: "1.0.0".to_string(),
                    file: "mylib-1.0.0.tgz".to_string(),
                },
            ),
            (
                StoragePath::from("mylib/1.0.0"),
                GetPath::VersionInfo {
                    name: "mylib".to_string(),
                    version: "1.0.0".to_string(),
                },
            ),
            (
                StoragePath::from("mylib"),
                GetPath::GetPackageInfo {
                    name: "mylib".to_string(),
                },
            ),
            (
                StoragePath::from("npm-check-updates/-/npm-check-updates-11.0.3.tgz"),
                GetPath::GetTar {
                    name: "npm-check-updates".to_string(),
                    version: "11.0.3".to_string(),
                    file: "npm-check-updates-11.0.3.tgz".to_string(),
                },
            ),
        ];
        for (path, expected) in tests {
            let get_path = GetPath::try_from(path).unwrap();
            assert_eq!(get_path, expected);
        }
    }
}

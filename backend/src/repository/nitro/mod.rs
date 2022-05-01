use std::error::Error;
use std::fmt::{Display, Formatter};
use actix_web::HttpRequest;
use crate::repository::response::{Project, RepoResponse};
use crate::storage::models::{Storage, StorageFile};
use serde::{Deserialize, Serialize};
use async_trait::async_trait;
use sea_orm::DatabaseConnection;
use crate::repository;
use crate::repository::data::{RepositoryConfig, RepositorySetting, RepositoryValue};
use crate::repository::nitro::NitroError::ProjectNotFound;
use crate::repository::response::RepoResponse::ProjectResponse;
use crate::repository::utils::get_version_data;
use crate::utils::get_current_time;

#[derive(Debug)]
pub enum NitroError {
    InternalError(String),
    ProjectNotFound,
}


impl From<&str> for NitroError {
    fn from(err: &str) -> NitroError {
        NitroError::InternalError(err.to_string())
    }
}

impl From<String> for NitroError {
    fn from(err: String) -> NitroError {
        NitroError::InternalError(err.to_string())
    }
}

#[async_trait]
pub trait NitroRepository<T: RepositorySetting> {
    fn parse_project_to_directory<S: Into<String>>(value: S) -> String;
    /// Handles a List of versions request
    async fn handle_versions(
        repository: &RepositoryConfig<T>,
        storage: &Storage,
        project: &str,
    ) -> Result<NitroRepoVersions, NitroError> {
        Ok(repository::utils::get_versions(&storage, &repository, Self::parse_project_to_directory(project)).await?)
    }
    async fn handle_version(
        repository: &RepositoryConfig<T>,
        storage: &Storage,
        project: &str,
        version: &str,
    ) -> Result<Project, NitroError> {
        let project_dir = Self::parse_project_to_directory(&project);

        let project_data =
            repository::utils::get_project_data(&storage, &repository, project_dir.clone()).await?;
        if let Some(project_data) = project_data {
            let version_data = get_version_data(
                &storage,
                &repository,
                format!("{}/{}", project_dir, &version),
            )
                .await?;

            let project = Project {
                repo_summary: repository.init_values.clone(),
                project: project_data,
                version: version_data,
                frontend_response: None,
            };
            return Ok(project);
        }
        return Err(ProjectNotFound);
    }
    async fn handle_project(
        repository: &RepositoryConfig<T>,
        storage: &Storage,
        project: &str,
    ) -> Result<Project, NitroError> {
        let project_dir = Self::parse_project_to_directory(&project);

        let project_data =
            repository::utils::get_project_data(&storage, &repository, project_dir.clone()).await?;
        if let Some(project_data) = project_data {
            let version_data = get_version_data(
                &storage,
                &repository,
                format!("{}/{}", &project_dir, &project_data.versions.latest_version),
            )
                .await?;

            let project = Project {
                repo_summary: repository.init_values.clone(),
                project: project_data,
                version: version_data,
                frontend_response: None,
            };
            return Ok(project);
        }
        return Err(ProjectNotFound);
    }

    /// Returns the latest version published.
    async fn latest_version(
        repository: &RepositoryConfig<T>,
        storage: &Storage,
        project: &str,
    ) -> Result<String, NitroError> {
        let project_dir = Self::parse_project_to_directory(&project);
        let project_data = repository::utils::get_project_data(&storage, &repository, project_dir).await?;
        if let Some(project_data) = project_data {
            let latest_release = project_data.versions.latest_release;
            if latest_release.is_empty() {
                Ok(project_data.versions.latest_version)
            } else {
                Ok(latest_release)
            }
        } else {
            Err(ProjectNotFound)
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct NitroFileResponse {
    pub files: Vec<NitroFile>,
    pub response_type: ResponseType,
    pub active_dir: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum ResponseType {
    Project(Option<Project>),
    Version(VersionBrowseResponse),
    Repository(RepositoryValue),
    Storage,
    Other,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct VersionBrowseResponse {
    pub project: Option<Project>,
    pub version: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct NitroFile {
    pub response_type: ResponseType,
    #[serde(flatten)]
    pub file: StorageFile,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RepositoryListing {
    pub values: Vec<String>,
}

impl RepositoryListing {
    pub fn add_value(&mut self, project: String) -> bool {
        for v in &self.values {
            if v.eq(&project) {
                return false;
            }
        }
        self.values.push(project);
        true
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VersionData {
    pub name: String,
    #[serde(default)]
    pub description: String,
    pub source: Option<ProjectSource>,
    pub licence: Option<Licence>,
    pub version: String,
    #[serde(default = "crate::utils::get_current_time")]
    pub created: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProjectData {
    #[serde(default)]
    pub versions: NitroRepoVersions,
    #[serde(default = "crate::utils::get_current_time")]
    pub created: i64,
    #[serde(default = "crate::utils::get_current_time")]
    pub updated: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProjectSource {
    pub name: String,
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Licence {}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NitroRepoVersions {
    #[serde(default)]
    pub latest_version: String,
    #[serde(default)]
    pub latest_release: String,
    pub versions: Vec<NitroVersion>,
}

impl Default for NitroRepoVersions {
    fn default() -> Self {
        NitroRepoVersions {
            latest_version: "".to_string(),
            latest_release: "".to_string(),
            versions: vec![],
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NitroVersion {
    pub version: String,
    pub time: i64,
    pub snapshot: bool,
}

impl From<String> for NitroVersion {
    fn from(value: String) -> Self {
        let x = value.contains("-SNAPSHOT");
        NitroVersion {
            version: value,
            time: 0,
            snapshot: x,
        }
    }
}

impl NitroRepoVersions {
    pub fn update_version(&mut self, version: String) {
        for v in self.versions.iter_mut() {
            if v.version.eq(&version) {
                if !v.snapshot {
                    v.time = get_current_time();
                }
                return;
            }
        }
        let snapshot = version.contains("-SNAPSHOT");
        // TODO encourage a consistent version standard.
        if snapshot {
            self.latest_version = version.clone();
        } else {
            self.latest_version = version.clone();
            self.latest_release = version.clone();
        }
        self.versions.push(NitroVersion {
            version,
            time: get_current_time(),
            snapshot,
        })
    }
    pub fn get(&self, version: &str) -> Option<NitroVersion> {
        for x in &self.versions {
            if x.version.eq(version) {
                return Some(x.clone());
            }
        }
        None
    }
}

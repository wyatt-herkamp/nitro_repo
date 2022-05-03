pub mod error;
pub mod handler;
pub mod nitro_repository;
pub mod utils;

use crate::repository::data::RepositoryValue;
use crate::repository::response::Project;
use crate::storage::models::StorageFile;
use crate::utils::get_current_time;
use serde::{Deserialize, Serialize};

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
impl Default for ProjectData {
    fn default() -> Self {
        ProjectData {
            versions: Default::default(),
            created: get_current_time(),
            updated: get_current_time(),
        }
    }
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
    pub fn update_version<S: Into<String>>(&mut self, version: S) {
        let version = version.into();
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

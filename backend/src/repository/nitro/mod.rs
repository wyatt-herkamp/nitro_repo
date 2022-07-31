use serde::{Deserialize, Serialize};

use crate::repository::response::Project;
use crate::storage::file::StorageFile;
use crate::utils::get_current_time;

pub mod dynamic;
pub mod nitro_repository;
pub mod utils;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectRequest {
    pub storage: String,
    pub repository: String,
    pub project_name: String,
    pub version: Option<String>,
}
impl ProjectRequest {
    pub fn into_inner(self) -> (String, String, String, Option<String>) {
        (
            self.storage,
            self.repository,
            self.project_name,
            self.version,
        )
    }
}
#[derive(Serialize, Clone, Debug)]
pub struct NitroFileResponse {
    pub files: Vec<NitroFile>,
    pub response_type: NitroFileResponseType,
    pub active_dir: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum NitroFileResponseType {
    Project(Project),
    Other,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct VersionBrowseResponse {
    pub project: Option<Project>,
    pub version: String,
}

#[derive(Serialize, Clone, Debug)]
pub struct NitroFile {
    pub response_type: NitroFileResponseType,
    #[serde(flatten)]
    pub file: StorageFile,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RepositoryListing {
    pub projects: Vec<String>,
    pub last_updated: i64,
}

impl RepositoryListing {
    pub fn add_value(&mut self, project: String) -> bool {
        for v in &self.projects {
            if v.eq(&project) {
                return false;
            }
        }
        self.projects.push(project);
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

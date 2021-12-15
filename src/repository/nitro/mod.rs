use serde::{Deserialize, Serialize};

use crate::utils::get_current_time;

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
        self.values.push(project.clone());
        return true;
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProjectData {
    #[serde(default = "crate::utils::get_current_time")]
    pub created: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NitroMavenVersions {
    #[serde(default)]
    pub latest_version: String,
    #[serde(default)]
    pub latest_release: String,
    pub versions: Vec<NitroVersion>,
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
        return NitroVersion {
            version: value,
            time: 0,
            snapshot: x,
        };
    }
}

impl NitroMavenVersions {
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
}

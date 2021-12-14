use serde::{Deserialize, Serialize};

use crate::utils::get_current_time;

#[derive(Debug, Serialize, Deserialize)]
pub struct DeployMetadata {
    #[serde(rename = "groupId")]
    pub group_id: String,
    #[serde(rename = "artifactId")]
    pub artifact_id: String,
    pub versioning: Versioning,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Versioning {
    pub release: Option<String>,
    pub versions: Versions,
    #[serde(rename = "lastUpdated")]
    pub last_updated: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Versions {
    pub version: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Pom {
    #[serde(rename = "groupId")]
    pub group_id: String,
    #[serde(rename = "artifactId")]
    pub artifact_id: String,
    pub version: String,
    pub name: Option<String>,
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
        self.values.push(project.clone());
        return true;
    }
}


use std::collections::HashMap;
use std::fs::{create_dir_all, File, read_to_string, remove_file};
use std::io::Write;
use std::path::PathBuf;

use chrono::{DateTime, NaiveDateTime, Utc};

use crate::error::internal_error::InternalError;
use crate::repository::models::Repository;
use crate::repository::nitro::{NitroMavenVersions, ProjectData};
use crate::repository::repository::VersionResponse;
use crate::repository::utils::get_versions;
use crate::storage::models::Storage;
use crate::utils::{get_current_time, get_storage_location};

static NPM_TIME_FORMAT: &'static str = "%Y-%m-%dT%H:%M:%S.%3fZ";

impl From<NitroMavenVersions> for HashMap<String, String> {
    fn from(value: NitroMavenVersions) -> Self {
        let mut map = HashMap::new();
        for x in value.versions {
            let naive = NaiveDateTime::from_timestamp(x.time, 0);
            let date_time: DateTime<Utc> = DateTime::from_utc(naive, Utc);
            let format = date_time.format(NPM_TIME_FORMAT).to_string();
            map.insert(x.version, format);
        }
        map
    }
}

pub fn get_version(path: &PathBuf, version: String) -> Option<VersionResponse> {
    let versions_value = get_versions(path);
    return get_version_by_data(&versions_value, version);
}
pub fn get_time_file<S: Into<String>>(storage: &Storage, repo: &Repository, id: S) -> PathBuf {
    get_storage_location()
        .join("storages")
        .join(&storage.name)
        .join(&repo.name)
        .join(id.into())
        .join("times.json")
}

pub fn update_project(project_folder: &PathBuf, version: String) -> Result<(), InternalError> {
    let buf = project_folder.join(".nitro.project.json");

    let mut project_data: ProjectData = if buf.exists() {
        let value = serde_json::from_str(&read_to_string(&buf)?).unwrap();
        remove_file(&buf)?;
        value
    } else {
        //TODO Pull NPM Data
        ProjectData {
            name: "".to_string(),
            description: "".to_string(),
            source: None,
            licence: None,
            versions: Default::default(),
            created: get_current_time(),
        }
    };
    project_data.versions.update_version(version);
    let mut file = File::create(&buf).unwrap();
    let string = serde_json::to_string_pretty(&project_data)?;
    let x1 = string.as_bytes();
    file.write_all(x1)?;
    return Ok(());
}

pub fn get_version_by_data(
    versions_value: &NitroMavenVersions,
    version: String,
) -> Option<VersionResponse> {
    for x in &versions_value.versions {
        if x.version.eq(&version) {
            return Some(VersionResponse {
                version: x.clone(),
                other: Default::default(),
            });
        }
    }
    return None;
pub fn read_time_file<S: Into<String>>(
    storage: &Storage,
    repo: &Repository,
    id: S,
) -> Result<HashMap<String, String>, InternalError> {
    let times_json = get_time_file(storage, repo, id);
    let times_map: HashMap<String, String> = serde_json::from_reader(File::open(&times_json)?)?;
    Ok(times_map)
}

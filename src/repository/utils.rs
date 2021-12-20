use std::fs::{File, read_to_string, remove_file};
use std::io::Write;
use std::path::PathBuf;

use crate::error::internal_error::InternalError;
use crate::repository::nitro::{NitroMavenVersions, ProjectData, RepositoryListing};
use crate::repository::repository::RepositoryRequest;
use crate::utils::{get_current_time, get_storage_location};

pub fn build_artifact_directory(request: &RepositoryRequest) -> PathBuf {
    return build_directory(&request).join(&request.value);
}

pub fn build_directory(request: &RepositoryRequest) -> PathBuf {
    return get_storage_location()
        .join("storages")
        .join(&request.storage.name)
        .join(&request.repository.name);
}

pub fn update_project_in_repositories(
    project: String,
    repo_location: PathBuf,
) -> Result<(), InternalError> {
    let buf = repo_location.join("repository.json");

    let mut repo_listing: RepositoryListing = if buf.exists() {
        let value = serde_json::from_str(&read_to_string(&buf)?).unwrap();
        value
    } else {
        RepositoryListing { values: vec![] }
    };

    if !repo_listing.add_value(project) && buf.exists() {
        remove_file(&buf)?;
    }
    let mut file = File::create(&buf).unwrap();
    let string = serde_json::to_string_pretty(&repo_listing)?;
    let x1 = string.as_bytes();
    file.write_all(x1)?;
    return Ok(());
}

pub fn get_versions(path: &PathBuf) -> NitroMavenVersions {
    let versions = path.join(".nitro.project.json");
    return if versions.exists() {
        serde_json::from_str(&read_to_string(&versions).unwrap()).unwrap()
    } else {
        ProjectData {
            versions: Default::default(),
            created: 0,
        }
    }
        .versions;
}

pub fn get_latest_version(path: &PathBuf, release: bool) -> Option<String> {
    let versions = path.join(".nitro.versions.json");
    return if versions.exists() {
        let option: NitroMavenVersions =
            serde_json::from_str(&read_to_string(&versions).unwrap()).unwrap();
        get_latest_version_data(&option, release)
    } else {
        None
    };
}

pub fn get_latest_version_data(
    versions_value: &NitroMavenVersions,
    release: bool,
) -> Option<String> {
    if release {
        Some(versions_value.latest_release.clone())
    } else {
        Some(versions_value.latest_version.clone())
    }
}

pub fn update_project(project_folder: &PathBuf, version: String) -> Result<(), InternalError> {
    let buf = project_folder.join(".nitro.project.json");

    let mut project_data: ProjectData = if buf.exists() {
        let value = serde_json::from_str(&read_to_string(&buf)?).unwrap();
        remove_file(&buf)?;
        value
    } else {
        ProjectData {
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

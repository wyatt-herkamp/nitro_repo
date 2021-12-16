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

pub fn update_versions(project_folder: &PathBuf, version: String) -> Result<(), InternalError> {
    let versions = project_folder.join(".nitro.versions.json");
    let mut versions_value: NitroMavenVersions = if versions.exists() {
        let value = serde_json::from_str(&read_to_string(&versions)?).unwrap();
        remove_file(&versions)?;
        value
    } else {
        NitroMavenVersions {
            latest_version: "".to_string(),
            latest_release: "".to_string(),
            versions: vec![],
        }
    };

    versions_value.update_version(version);
    let mut file = File::create(&versions).unwrap();
    let string = serde_json::to_string_pretty(&versions_value)?;
    let x1 = string.as_bytes();
    file.write_all(x1)?;
    return Ok(());
}

pub fn update_project(project_folder: &PathBuf) -> Result<(), InternalError> {
    let buf = project_folder.join(".nitro.project.json");

    let repo_listing: ProjectData = if buf.exists() {
        let value = serde_json::from_str(&read_to_string(&buf)?).unwrap();
        remove_file(&buf)?;
        value
    } else {
        ProjectData {
            created: get_current_time(),
        }
    };
    //TODO append updates
    let mut file = File::create(&buf).unwrap();
    let string = serde_json::to_string_pretty(&repo_listing)?;
    let x1 = string.as_bytes();
    file.write_all(x1)?;
    return Ok(());
}

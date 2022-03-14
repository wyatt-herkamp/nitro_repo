use crate::error::internal_error::InternalError;
use crate::repository::nitro::{NitroMavenVersions, ProjectData, RepositoryListing};
use crate::repository::types::RepositoryRequest;
use std::fs::{read_to_string, remove_file, File};
use std::io::Write;
use std::path::{Path, PathBuf};

use crate::utils::get_storage_location;

pub fn build_artifact_directory(request: &RepositoryRequest) -> PathBuf {
    build_directory(request).join(&request.value)
}

pub fn build_directory(request: &RepositoryRequest) -> PathBuf {
    get_storage_location()
        .join("storages")
        .join(&request.storage.name)
        .join(&request.repository.name)
}

pub fn update_project_in_repositories(
    project: String,
    repo_location: PathBuf,
) -> Result<(), InternalError> {
    let buf = repo_location.join("repository.json");

    let mut repo_listing: RepositoryListing = if buf.exists() {
        
        serde_json::from_str(&read_to_string(&buf)?).unwrap()
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
    Ok(())
}

pub fn get_versions(path:  &Path) -> NitroMavenVersions {
    let versions = path.join(".nitro.project.json");
    if versions.exists() {
        let data: ProjectData = serde_json::from_str(&read_to_string(&versions).unwrap()).unwrap();
        data.versions
    } else {
        Default::default()
    }
}

pub fn get_latest_version(path: &Path, release: bool) -> Option<String> {
    let versions = path.join(".nitro.versions.json");
    if versions.exists() {
        let option: NitroMavenVersions =
            serde_json::from_str(&read_to_string(&versions).unwrap()).unwrap();
        get_latest_version_data(&option, release)
    } else {
        None
    }
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
pub fn get_project_data(path: &Path) -> Result<Option<ProjectData>, InternalError> {
    let buf = path.join(".nitro.project.json");
    if !buf.exists() {
        return Ok(None);
    }
    Ok(Some(serde_json::from_str(&read_to_string(buf)?).unwrap()))
}
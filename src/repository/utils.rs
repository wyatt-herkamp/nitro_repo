use crate::constants::{PROJECTS_FILE, PROJECT_FILE};
use crate::error::internal_error::InternalError;
use crate::repository::models::Repository;
use crate::repository::nitro::{NitroMavenVersions, ProjectData, RepositoryListing};
use crate::repository::types::RepositoryRequest;
use crate::storage::models::StringStorage;
use std::fs::read_to_string;
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
    storage: &StringStorage,
    repository: &Repository,
    project: String,
) -> Result<(), InternalError> {
    let option = storage.get_file(repository, PROJECTS_FILE)?;
    let mut repo_listing: RepositoryListing = if let Some(data) = option {
        let data = String::from_utf8(data)?;
        storage.delete_file(repository, PROJECTS_FILE)?;
        serde_json::from_str(&data)?
    } else {
        RepositoryListing { values: vec![] }
    };

    repo_listing.add_value(project);
    let string = serde_json::to_string_pretty(&repo_listing)?;
    storage.save_file(repository, string.as_bytes(), PROJECTS_FILE)?;
    Ok(())
}

pub fn get_versions(
    storage: &StringStorage,
    repository: &Repository,
    path: String,
) -> Result<NitroMavenVersions, InternalError> {
    let string = format!("{}/{}", path, PROJECT_FILE);
    let option = storage.get_file(repository, &string)?;
    Ok(if let Some(vec) = option {
        let data: ProjectData = serde_json::from_str(&String::from_utf8(vec)?)?;
        data.versions
    } else {
        Default::default()
    })
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

pub fn get_project_data(
    storage: &StringStorage,
    repository: &Repository,
    project: String,
) -> Result<Option<ProjectData>, InternalError> {
    let string = format!("{}/{}", project, PROJECT_FILE);
    let option = storage.get_file(repository, &string)?;
    Ok(if let Some(vec) = option {
        let data: ProjectData = serde_json::from_str(&String::from_utf8(vec)?)?;
        Some(data)
    } else {
        None
    })
}

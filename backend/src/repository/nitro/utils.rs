use std::fs::read_to_string;
use std::path::Path;

use log::debug;

use crate::constants::{PROJECTS_FILE, PROJECT_FILE, VERSION_DATA};
use crate::error::internal_error::InternalError;
use crate::repository::data::RepositoryConfig;
use crate::repository::nitro::{NitroRepoVersions, ProjectData, RepositoryListing, VersionData};
use crate::repository::response::VersionResponse;
use crate::storage::models::Storage;
use crate::storage::DynamicStorage;

pub async fn get_version<StorageType: Storage>(
    storage: &StorageType,
    repository: &RepositoryConfig,
    project: String,
    version: String,
) -> Result<Option<VersionResponse>, InternalError> {
    let versions_value = get_versions(storage, repository, project).await?;
    Ok(get_version_by_data(&versions_value, version))
}

pub fn get_version_by_data(
    versions_value: &NitroRepoVersions,
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
    None
}

pub async fn update_project_in_repositories<StorageType: Storage>(
    storage: &StorageType,
    repository: &RepositoryConfig,
    project: String,
) -> Result<(), InternalError> {
    let option = storage.get_file(repository, PROJECTS_FILE).await?;
    let mut repo_listing: RepositoryListing = if let Some(data) = option {
        let data = String::from_utf8(data)?;
        storage.delete_file(repository, PROJECTS_FILE).await?;
        serde_json::from_str(&data)?
    } else {
        RepositoryListing { values: vec![] }
    };

    repo_listing.add_value(project);
    let string = serde_json::to_string_pretty(&repo_listing)?;
    storage
        .save_file(repository, string.as_bytes(), PROJECTS_FILE)
        .await?;
    Ok(())
}

pub async fn get_versions<StorageType: Storage>(
    storage: &StorageType,
    repository: &RepositoryConfig,
    path: String,
) -> Result<NitroRepoVersions, InternalError> {
    let string = format!("{}/{}", path, PROJECT_FILE);
    let option = storage.get_file(repository, &string).await?;
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
        let option: NitroRepoVersions =
            serde_json::from_str(&read_to_string(&versions).unwrap()).unwrap();
        get_latest_version_data(&option, release)
    } else {
        None
    }
}

pub fn get_latest_version_data(
    versions_value: &NitroRepoVersions,
    release: bool,
) -> Option<String> {
    if release {
        Some(versions_value.latest_release.clone())
    } else {
        Some(versions_value.latest_version.clone())
    }
}

pub async fn get_project_data<StorageType: Storage>(
    storage: &StorageType,
    repository: &RepositoryConfig,
    project: &str,
) -> Result<Option<ProjectData>, InternalError> {
    let string = format!("{}/{}", project, PROJECT_FILE);
    debug!("Project Data Location {}", &string);
    let option = storage.get_file(repository, &string).await?;
    Ok(if let Some(vec) = option {
        let mut data: ProjectData = serde_json::from_str(&String::from_utf8(vec)?)?;
        if data.versions.latest_release.is_empty() {
            data.versions.latest_release = data.versions.latest_version.clone();
        }
        Some(data)
    } else {
        None
    })
}

pub async fn get_version_data<StorageType: Storage>(
    storage: &StorageType,
    repository: &RepositoryConfig,
    folder: &str,
) -> Result<Option<VersionData>, InternalError> {
    let string = format!("{}/{}", folder, VERSION_DATA);
    debug!("Version Data Location {}", &string);
    let option = storage.get_file(repository, &string).await?;
    Ok(if let Some(vec) = option {
        let data: VersionData = serde_json::from_str(&String::from_utf8(vec)?)?;

        Some(data)
    } else {
        None
    })
}

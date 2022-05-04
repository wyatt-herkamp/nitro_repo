use crate::constants::{PROJECTS_FILE, PROJECT_FILE, VERSION_DATA};
use crate::repository::data::RepositoryConfig;
use crate::repository::nitro::error::NitroError;
use crate::repository::nitro::{
    NitroFile, NitroFileResponse, NitroRepoVersions, ProjectData, RepositoryListing, ResponseType,
    VersionData,
};
use crate::repository::response::{Project, VersionResponse};
use crate::storage::models::{Storage, StorageFile};
use log::debug;
use std::fs::read_to_string;
use std::path::Path;

pub async fn process_storage_files(
    storage: &Box<dyn Storage>,
    repo: &RepositoryConfig,
    storage_files: Vec<StorageFile>,
    requested_dir: &str,
) -> Result<NitroFileResponse, NitroError> {
    let mut nitro_files = Vec::new();
    for file in storage_files {
        nitro_files.push(NitroFile {
            //TODO Implement This
            response_type: ResponseType::Other,
            file,
        });
    }
    let active_dir = format!(
        "{}/{}/{}",
        &storage.config_for_saving().generic_config.name,
        &repo.name,
        requested_dir
    );
    let string = format!("{}/{}", &requested_dir, PROJECT_FILE);
    let option = storage.get_file(repo, &string).await?;
    return if let Some(data) = option {
        let mut data: ProjectData = serde_json::from_slice(data.as_slice())?;
        if data.versions.latest_release.is_empty() {
            data.versions.latest_release = data.versions.latest_version.clone();
        }
        let version_data = get_version_data(
            storage,
            repo,
            format!("{}/{}", requested_dir, data.versions.latest_release),
        )
        .await?;
        let project = Project {
            repo_summary: repo.clone(),
            project: data,
            version: version_data,
            frontend_response: None,
        };
        Ok(NitroFileResponse {
            files: nitro_files,
            response_type: ResponseType::Project(Some(project)),
            active_dir,
        })
    } else {
        let string = format!("{}/{}", &requested_dir, VERSION_DATA);
        let option = storage.get_file(repo, &string).await?;

        if let Some(version) = option {
            let version: VersionData = serde_json::from_slice(version.as_slice())?;

            let x = Path::new(&requested_dir).parent().unwrap();
            let string = format!("{}/{}", x.to_str().unwrap(), PROJECT_FILE);
            let option = storage.get_file(repo, &string).await?;

            let mut project_data: ProjectData = serde_json::from_slice(option.unwrap().as_slice())?;
            if project_data.versions.latest_release.is_empty() {
                project_data.versions.latest_release = project_data.versions.latest_version.clone();
            }
            let project = Project {
                repo_summary: repo.clone(),
                project: project_data,
                version: Some(version),
                frontend_response: None,
            };

            Ok(NitroFileResponse {
                files: nitro_files,
                active_dir,

                response_type: ResponseType::Project(Some(project)),
            })
        } else {
            Ok(NitroFileResponse {
                active_dir,
                files: nitro_files,
                response_type: ResponseType::Repository(repo.clone()),
            })
        }
    };
}
pub async fn get_version(
    storage: &Box<dyn Storage>,
    repository: &RepositoryConfig,
    project: String,
    version: String,
) -> Result<Option<VersionResponse>, NitroError> {
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

pub async fn update_project_in_repositories(
    storage: &Box<dyn Storage>,
    repository: &RepositoryConfig,
    project: String,
) -> Result<(), NitroError> {
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

pub async fn get_versions(
    storage: &Box<dyn Storage>,
    repository: &RepositoryConfig,
    path: String,
) -> Result<NitroRepoVersions, NitroError> {
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

pub async fn get_project_data(
    storage: &Box<dyn Storage>,
    repository: &RepositoryConfig,
    project: String,
) -> Result<Option<ProjectData>, NitroError> {
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
pub async fn get_version_data(
    storage: &Box<dyn Storage>,
    repository: &RepositoryConfig,
    folder: String,
) -> Result<Option<VersionData>, NitroError> {
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

use crate::repository::response::Project;
use crate::storage::models::Storage;

use crate::constants::{PROJECT_FILE, VERSION_DATA};
use async_trait::async_trait;
use log::{debug, error, trace};

use crate::repository::data::{RepositoryConfig, RepositorySetting};
use crate::repository::nitro::error::NitroError;
use crate::repository::nitro::error::NitroError::ProjectNotFound;
use crate::repository::nitro::utils::{
    get_project_data, get_version_data, get_versions, update_project_in_repositories,
};
use crate::repository::nitro::{NitroRepoVersions, ProjectData, VersionData};
use crate::system::user::UserModel;
use crate::utils::get_current_time;

#[async_trait]
pub trait NitroRepository<T: RepositorySetting> {
    fn parse_project_to_directory<S: Into<String>>(value: S) -> String;
    /// Handles a List of versions request
    async fn handle_versions(
        repository: &RepositoryConfig<T>,
        storage: &Storage,
        project: &str,
    ) -> Result<NitroRepoVersions, NitroError> {
        Ok(get_versions(
            storage,
            &repository,
            Self::parse_project_to_directory(project),
        )
        .await?)
    }
    async fn handle_version(
        repository: &RepositoryConfig<T>,
        storage: &Storage,
        project: &str,
        version: &str,
    ) -> Result<Project, NitroError> {
        let project_dir = Self::parse_project_to_directory(project);

        let project_data = get_project_data(storage, &repository, project_dir.clone()).await?;
        if let Some(project_data) = project_data {
            let version_data = get_version_data(
                storage,
                &repository,
                format!("{}/{}", project_dir, &version),
            )
            .await?;

            let project = Project {
                repo_summary: repository.init_values.clone(),
                project: project_data,
                version: version_data,
                frontend_response: None,
            };
            return Ok(project);
        }
        return Err(ProjectNotFound);
    }
    async fn handle_project(
        repository: &RepositoryConfig<T>,
        storage: &Storage,
        project: &str,
    ) -> Result<Project, NitroError> {
        let project_dir = Self::parse_project_to_directory(project);

        let project_data = get_project_data(storage, &repository, project_dir.clone()).await?;
        if let Some(project_data) = project_data {
            let version_data = get_version_data(
                storage,
                &repository,
                format!("{}/{}", &project_dir, &project_data.versions.latest_version),
            )
            .await?;

            let project = Project {
                repo_summary: repository.init_values.clone(),
                project: project_data,
                version: version_data,
                frontend_response: None,
            };
            return Ok(project);
        }
        return Err(ProjectNotFound);
    }

    /// Returns the latest version published.
    async fn latest_version(
        repository: &RepositoryConfig<T>,
        storage: &Storage,
        project: &str,
    ) -> Result<String, NitroError> {
        let project_dir = Self::parse_project_to_directory(project);
        let project_data = get_project_data(storage, &repository, project_dir).await?;
        if let Some(project_data) = project_data {
            let latest_release = project_data.versions.latest_release;
            if latest_release.is_empty() {
                Ok(project_data.versions.latest_version)
            } else {
                Ok(latest_release)
            }
        } else {
            Err(ProjectNotFound)
        }
    }
    /// Handles the Update Process and Post Deploy tasks for a Nitro Repository
    async fn post_deploy(
        storage: &Storage,
        repository: &RepositoryConfig<T>,
        project_folder: String,
        version_folder: String,
        _: UserModel,
        version_data: VersionData,
    ) -> Result<(), NitroError> {
        let project_file = format!("{}/{}", &project_folder, PROJECT_FILE);
        let version_file = format!("{}/{}", &version_folder, VERSION_DATA);
        trace!("Project File Location {}", project_file);
        trace!("Version File Location {}", version_file);
        let result: Result<(), NitroError> = {
            let option = storage.get_file(&repository, &project_file).await?;
            let mut project_data: ProjectData = if let Some(data) = option {
                let string = String::from_utf8(data)?;
                let value = serde_json::from_str(&string)?;
                storage.delete_file(&repository, &project_file).await?;
                value
            } else {
                debug!("Creating new Project Data Value");
                ProjectData::default()
            };
            project_data.versions.update_version(&version_data.version);
            storage
                .save_file(
                    &repository,
                    serde_json::to_string_pretty(&project_data)?.as_bytes(),
                    &project_file,
                )
                .await?;
            storage
                .save_file(
                    &repository,
                    serde_json::to_string_pretty(&version_data)?.as_bytes(),
                    &version_folder,
                )
                .await?;
            Ok(())
        };
        if let Err(error) = result {
            error!("Unable to update {}, {}", PROJECT_FILE, error);
            trace!(
                "Version {} Name: {}",
                &version_data.version,
                &version_data.name
            );
        }
        if let Err(error) = update_project_in_repositories(
            &storage,
            &repository.init_values,
            version_data.name.clone(),
        )
        .await
        {
            error!("Unable to update repository.json, {}", error);
            trace!(
                "Version {} Name: {}",
                &version_data.version,
                &version_data.name
            );
        }
        Ok(())
    }
}

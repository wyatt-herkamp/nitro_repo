use crate::repository::response::Project;
use crate::storage::models::Storage;

use async_trait::async_trait;

use crate::repository::data::{RepositoryConfig, RepositorySetting};
use crate::repository::nitro::error::NitroError;
use crate::repository::nitro::error::NitroError::ProjectNotFound;
use crate::repository::nitro::utils::{get_project_data, get_version_data, get_versions};
use crate::repository::nitro::NitroRepoVersions;

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
}

use async_trait::async_trait;
use log::{debug, error, trace};

use crate::constants::{PROJECT_FILE, VERSION_DATA};
use crate::error::internal_error::InternalError;
use crate::repository::nitro::utils::{
    get_project_data, get_version_data, get_versions, update_project_in_repositories,
};
use crate::repository::nitro::{
    NitroFile, NitroFileResponse, NitroFileResponseType, NitroRepoVersions, ProjectData,
    VersionData,
};
use crate::repository::response::Project;
use crate::repository::settings::RepositoryConfig;
use crate::storage::file::StorageDirectoryResponse;
use crate::storage::models::Storage;
use crate::system::user::UserModel;

#[async_trait]
pub trait NitroRepositoryHandler<StorageType: Storage> {
    fn parse_project_to_directory<S: Into<String>>(value: S) -> String;
    fn storage(&self) -> &StorageType;
    fn repository(&self) -> &RepositoryConfig;
    /// Handles a List of versions request
    async fn get_versions(
        &self,
        project: &str,
    ) -> Result<Option<NitroRepoVersions>, InternalError> {
        Ok(Some(
            get_versions(
                self.storage(),
                self.repository(),
                Self::parse_project_to_directory(project),
            )
            .await?,
        ))
    }
    async fn get_project_specific_version(
        &self,
        project: &str,
        version: &str,
    ) -> Result<Option<Project>, InternalError> {
        let project_dir = Self::parse_project_to_directory(project);

        let project_data =
            get_project_data(self.storage(), self.repository(), &project_dir.clone()).await?;
        if let Some(project_data) = project_data {
            let version_data = get_version_data(
                self.storage(),
                self.repository(),
                &format!("{}/{}", project_dir, &version),
            )
            .await?;

            let project = Project {
                repo_summary: self.repository().clone(),
                project: project_data,
                version: version_data,
                frontend_response: String::new(),
            };
            return Ok(Some(project));
        }
        return Ok(None);
    }
    async fn get_project_latest(&self, project: &str) -> Result<Option<Project>, InternalError> {
        let project_dir = Self::parse_project_to_directory(project);

        let project_data =
            get_project_data(self.storage(), self.repository(), &project_dir).await?;
        if let Some(project_data) = project_data {
            let version_data = get_version_data(
                self.storage(),
                self.repository(),
                &format!("{}/{}", &project_dir, &project_data.versions.latest_version),
            )
            .await?;

            let project = Project {
                repo_summary: self.repository().clone(),
                project: project_data,
                version: version_data,
                frontend_response: String::new(),
            };
            return Ok(Some(project));
        }
        return Ok(None);
    }

    /// Returns the latest version published.
    async fn latest_version(&self, project: &str) -> Result<Option<String>, InternalError> {
        let project_dir = Self::parse_project_to_directory(project);
        let project_data =
            get_project_data(self.storage(), self.repository(), &project_dir).await?;
        if let Some(project_data) = project_data {
            let latest_release = project_data.versions.latest_release;
            if latest_release.is_empty() {
                Ok(Some(project_data.versions.latest_version))
            } else {
                Ok(Some(latest_release))
            }
        } else {
            Ok(None)
        }
    }

    async fn process_storage_files(
        &self,
        directory: StorageDirectoryResponse,
        requested_dir: &str,
    ) -> Result<NitroFileResponse, InternalError> {
        let mut nitro_files = Vec::new();
        for file in directory.files {
            nitro_files.push(NitroFile {
                //TODO Implement This
                response_type: NitroFileResponseType::Other,
                file,
            });
        }
        let value = if let Some(project_data) =
            get_project_data(self.storage(), self.repository(), requested_dir).await?
        {
            let version_data = get_version_data(
                self.storage(),
                self.repository(),
                &format!(
                    "{}/{}",
                    &requested_dir, &project_data.versions.latest_version
                ),
            )
            .await?;

            let project = Project {
                repo_summary: self.repository().clone(),
                project: project_data,
                version: version_data,
                frontend_response: String::new(),
            };
            NitroFileResponseType::Project(project)
        } else if let Some(version) =
            get_version_data(self.storage(), self.repository(), requested_dir).await?
        {
            let project_dir = Self::parse_project_to_directory(&version.name);
            let project = get_project_data(self.storage(), self.repository(), &project_dir)
                .await?
                .unwrap();

            let project = Project {
                repo_summary: self.repository().clone(),
                project,
                version: Some(version),
                frontend_response: String::new(),
            };
            NitroFileResponseType::Project(project)
        } else {
            NitroFileResponseType::Other
        };
        Ok(NitroFileResponse {
            files: nitro_files,
            response_type: value,
            active_dir: requested_dir.to_string(),
        })
    }

    /// Handles the Update Process and Post Deploy tasks for a Nitro Repository
    async fn post_deploy(
        &self,
        project_folder: String,
        version_folder: String,
        _: UserModel,
        version_data: VersionData,
    ) -> Result<(), InternalError> {
        let project_file = format!("{}/{}", &project_folder, PROJECT_FILE);
        let version_file = format!("{}/{}", &version_folder, VERSION_DATA);
        trace!("Project File Location {}", project_file);
        trace!("Version File Location {}", version_file);
        let result: Result<(), InternalError> = {
            let option = self
                .storage()
                .get_file(self.repository(), &project_file)
                .await?;
            let mut project_data: ProjectData = if let Some(data) = option {
                let string = String::from_utf8(data)?;
                let value = serde_json::from_str(&string)?;
                self.storage()
                    .delete_file(self.repository(), &project_file)
                    .await?;
                value
            } else {
                debug!("Creating new Project Data Value");
                ProjectData::default()
            };
            project_data.versions.update_version(&version_data.version);
            self.storage()
                .save_file(
                    self.repository(),
                    serde_json::to_string_pretty(&project_data)?.as_bytes(),
                    &project_file,
                )
                .await?;
            self.storage()
                .save_file(
                    self.repository(),
                    serde_json::to_string_pretty(&version_data)?.as_bytes(),
                    &version_file,
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
            self.storage(),
            self.repository(),
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

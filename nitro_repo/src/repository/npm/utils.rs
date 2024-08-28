use nr_core::{
    database::project::{DBProject, DBProjectVersion, ProjectDBType},
    storage::StoragePath,
};
use tracing::{info, instrument};

use crate::repository::Repository;

use super::{types::request::PublishVersion, NPMRegistryError};

pub mod npm_time {
    use chrono::{DateTime, FixedOffset};

    pub fn format_date_time(date_time: &DateTime<FixedOffset>) -> String {
        date_time.format("%Y-%m-%dT%H:%M:%S.%3fZ").to_string()
    }
}
pub trait NpmRegistryExt: Repository {
    #[instrument]
    async fn get_or_create_project(
        &self,
        save_path: &StoragePath,
        release: &PublishVersion,
    ) -> Result<DBProject, NPMRegistryError> {
        if let Some(project) = DBProject::find_by_project_key(
            &release.name.to_string(),
            self.id(),
            &self.site().as_ref(),
        )
        .await?
        {
            // TODO: Update
            return Ok(project);
        }

        match release.new_project(save_path.to_string(), self.id()) {
            Ok(ok) => {
                let insert = ok.insert(&self.site().as_ref()).await?;
                info!(?insert, "Created new project");
                Ok(insert)
            }
            Err(err) => {
                return Err(err);
            }
        }
    }
    #[instrument]
    async fn create_or_update_version(
        &self,
        publisher: i32,
        save_path: &StoragePath,
        project: &DBProject,
        release: &PublishVersion,
    ) -> Result<(), NPMRegistryError> {
        if let Some(version) = DBProjectVersion::find_by_version_and_project(
            &release.version,
            project.id,
            &self.site().database,
        )
        .await?
        {
            return Ok(());
        }

        match release.new_version(project.id, save_path.to_string(), publisher) {
            Ok(ok) => {
                ok.insert_no_return(&self.site().database).await?;
                return Ok(());
            }
            Err(err) => {
                return Err(err);
            }
        }
    }
}

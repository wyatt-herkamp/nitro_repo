use std::sync::Arc;

use nr_core::{
    database::entities::project::{
        DBProject, ProjectDBType, update::UpdateProject, versions::DBProjectVersion,
    },
    storage::StoragePath,
};
use tracing::{info, instrument};

use super::{
    NPMRegistryError, login::web_login::NpmWebLoginManager, types::request::PublishVersion,
};
use crate::repository::Repository;

pub mod npm_time {
    use chrono::{DateTime, FixedOffset};

    pub fn format_date_time(date_time: &DateTime<FixedOffset>) -> String {
        date_time.format("%Y-%m-%dT%H:%M:%S.%3fZ").to_string()
    }
}
pub trait NpmRegistryExt: Repository {
    /// The browser-login sessions this registry shares with the `/api/npm` routes.
    ///
    /// On the trait rather than read off the site: the manager belongs to `NpmRegistryType`, and
    /// the login handlers take `&impl NpmRegistryExt` rather than a concrete registry.
    fn web_logins(&self) -> &Arc<NpmWebLoginManager>;

    #[instrument]
    async fn get_or_create_project(
        &self,
        save_path: &StoragePath,
        release: &PublishVersion,
    ) -> Result<DBProject, NPMRegistryError> {
        if let Some(project) = DBProject::find_by_project_key(
            &release.name.to_string(),
            self.id(),
            self.site().as_ref(),
        )
        .await?
        {
            // A re-publish carrying a changed description used to be dropped on the floor — there
            // was a `// TODO: Update` here because `UpdateProject` was an empty file. The rest of
            // the row is derived from the package name and path, which cannot change without
            // being a different project.
            let description = release.description();
            let update = UpdateProject {
                description: (description != project.description.as_deref())
                    .then(|| description.map(str::to_owned)),
                ..Default::default()
            };
            if !update.is_empty() {
                update.update(project.id, self.site().as_ref()).await?;
            }
            return Ok(project);
        }

        let new_project = release.new_project(save_path.to_string(), self.id())?;
        let insert = new_project.insert(self.site().as_ref()).await?;
        info!(?insert, "Created new project");
        Ok(insert)
    }

    /// Records a newly published version.
    ///
    /// Re-publishing an existing version is refused, which is what npm's own registry does. It
    /// previously returned `Ok` without touching the database while the caller went on to
    /// overwrite the tarball anyway — so the stored file and the recorded metadata described
    /// different builds, and `dist.integrity` in the packument no longer matched what was served.
    #[instrument]
    async fn create_version(
        &self,
        publisher: i32,
        save_path: &StoragePath,
        project: &DBProject,
        release: &PublishVersion,
    ) -> Result<(), NPMRegistryError> {
        if DBProjectVersion::find_by_version_and_project(
            &release.version,
            project.id,
            &self.site().database,
        )
        .await?
        .is_some()
        {
            return Err(NPMRegistryError::VersionAlreadyExists {
                version: release.version.clone(),
            });
        }

        let new_version = release.new_version(project.id, save_path.to_string(), publisher)?;
        new_version.insert(&self.site().database).await?;
        Ok(())
    }
}

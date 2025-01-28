use nr_core::{
    database::entities::project::{versions::DBProjectVersion, DBProject, ProjectDBType},
    repository::Visibility,
    user::permissions::{HasPermissions, RepositoryActions},
};
use sqlx::PgPool;
use uuid::Uuid;

use super::{Repository, RepositoryHandlerError};

pub async fn can_read_repository<A: HasPermissions>(
    auth: &A,
    visibility: Visibility,
    repository_id: Uuid,
    database: &PgPool,
) -> Result<bool, sqlx::Error> {
    match visibility {
        Visibility::Public => Ok(true),
        Visibility::Private | Visibility::Hidden => Ok(auth
            .has_action(RepositoryActions::Read, repository_id, database)
            .await?),
    }
}
pub trait RepositoryExt: Repository {
    async fn get_project_from_key(
        &self,
        project_key: &str,
    ) -> Result<Option<DBProject>, RepositoryHandlerError> {
        let project =
            DBProject::find_by_project_key(project_key, self.id(), self.site().as_ref()).await?;
        Ok(project)
    }
    async fn get_project_version(
        &self,
        project: Uuid,
        version: &str,
    ) -> Result<Option<DBProjectVersion>, RepositoryHandlerError> {
        let version =
            DBProjectVersion::find_by_version_and_project(version, project, &self.site().database)
                .await?;
        Ok(version)
    }
}

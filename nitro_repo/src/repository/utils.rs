use nr_core::{
    repository::Visibility,
    user::permissions::{HasPermissions, RepositoryActionOptions},
};
use sqlx::PgPool;
use uuid::Uuid;

use crate::app::authentication::Authentication;

pub async fn can_read_repository(
    auth: Option<Authentication>,
    visibility: Visibility,
    repository_id: Uuid,
    database: &PgPool,
) -> Result<bool, sqlx::Error> {
    match visibility {
        Visibility::Public => Ok(true),
        Visibility::Private | Visibility::Hidden => Ok(auth
            .has_action(RepositoryActionOptions::Read, repository_id, database)
            .await?),
    }
}

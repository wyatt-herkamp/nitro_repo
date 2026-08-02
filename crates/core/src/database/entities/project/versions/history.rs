use chrono::{DateTime, FixedOffset};
use pgsmith::prelude::*;
use serde::Serialize;
use sqlx::prelude::FromRow;
use utoipa::ToSchema;
use uuid::Uuid;

use super::{DBProjectVersion, ProjectVersionType};
use crate::{
    database::{DBResult, entities::project::versions::DBProjectVersionColumn},
    repository::project::ReleaseType,
};
/// One entry of a project's publish history.
///
/// #506 asks for "Push/Publish History". The type existed and was already served, but without the
/// one thing a history is usually consulted for — who published it. `publisher` is the user id and
/// `publisher_username` is resolved alongside it, so the UI does not have to fetch a user per row.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, FromRow, ToSchema)]
pub struct VersionHistoryItem {
    pub id: Uuid,
    pub release_type: ReleaseType,
    pub version: String,
    pub publisher: Option<i32>,
    pub publisher_username: Option<String>,
    pub updated_at: DateTime<FixedOffset>,
    pub created_at: DateTime<FixedOffset>,
}
impl TableQuery for VersionHistoryItem {
    type Table = DBProjectVersion;

    fn columns() -> Vec<DBProjectVersionColumn>
    where
        Self: Sized,
    {
        vec![
            DBProjectVersionColumn::Id,
            DBProjectVersionColumn::ReleaseType,
            DBProjectVersionColumn::Version,
            DBProjectVersionColumn::UpdatedAt,
            DBProjectVersionColumn::CreatedAt,
        ]
    }
}
impl ProjectVersionType for VersionHistoryItem {
    fn id(&self) -> Uuid {
        self.id
    }
}

impl VersionHistoryItem {
    /// A project's versions, newest first, with the publisher resolved.
    ///
    /// Hand-written rather than built with `SelectQueryBuilder` because the builder has no way to
    /// express the left join onto `users` — and it has to be a *left* join, since a version
    /// published before the publisher was recorded, or by a user since deleted, still belongs in
    /// the history.
    pub async fn find_by_project_id(
        project_id: Uuid,
        database: &sqlx::PgPool,
    ) -> DBResult<Vec<Self>> {
        let versions = sqlx::query_as::<_, Self>(
            r#"SELECT project_versions.id,
                      project_versions.release_type,
                      project_versions.version,
                      project_versions.publisher,
                      users.username AS publisher_username,
                      project_versions.updated_at,
                      project_versions.created_at
               FROM project_versions
               LEFT JOIN users ON users.id = project_versions.publisher
               WHERE project_versions.project_id = $1
               ORDER BY project_versions.updated_at DESC"#,
        )
        .bind(project_id)
        .fetch_all(database)
        .await?;
        Ok(versions)
    }
}

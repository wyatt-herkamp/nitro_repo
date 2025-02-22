use crate::{
    database::prelude::*,
    repository::project::{ReleaseType, VersionData},
};
use sqlx::types::Json;
use uuid::Uuid;

use super::{DBProjectVersion, DBProjectVersionColumn};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewVersion {
    pub project_id: Uuid,
    /// The version of the project
    pub version: String,
    /// Release type
    pub release_type: ReleaseType,
    /// The path to the release
    pub version_path: String,
    /// The publisher of the version
    pub publisher: Option<i32>,
    /// The version page. Such as a README
    pub version_page: Option<String>,
    /// The version data. More data can be added in the future and the data can be repository dependent
    pub extra: VersionData,
}
impl NewVersion {
    pub async fn insert(self, db: &PgPool) -> Result<DBProjectVersion, sqlx::Error> {
        let Self {
            project_id,
            version,
            release_type,
            version_path,
            publisher,
            version_page,
            extra,
        } = self;
        let db_version = InsertQueryBuilder::new(DBProjectVersion::table_name())
            .insert(DBProjectVersionColumn::ProjectId, project_id.value())
            .insert(DBProjectVersionColumn::Version, version.value())
            .insert(DBProjectVersionColumn::ReleaseType, release_type.value())
            .insert(DBProjectVersionColumn::Path, version_path.value())
            .insert(DBProjectVersionColumn::Publisher, publisher.value())
            .insert(DBProjectVersionColumn::VersionPage, version_page.value())
            .insert(DBProjectVersionColumn::Extra, Json(extra).value())
            .return_all()
            .query_as()
            .fetch_one(db)
            .await?;

        Ok(db_version)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct UpdateProjectVersion {
    pub release_type: Option<ReleaseType>,
    pub publisher: Option<Option<i32>>,
    pub version_page: Option<Option<String>>,
    pub extra: Option<VersionData>,
}
impl UpdateProjectVersion {
    pub async fn update(&self, version_id: Uuid, database: &PgPool) -> DBResult<()> {
        let mut update = UpdateQueryBuilder::new(DBProjectVersion::table_name());
        update
            .set(DBProjectVersionColumn::Id, version_id)
            .set(DBProjectVersionColumn::UpdatedAt, SqlFunctionBuilder::now());

        if let Some(release_type) = &self.release_type {
            update.set(DBProjectVersionColumn::ReleaseType, release_type);
        }
        if let Some(extra) = &self.extra {
            update.set(DBProjectVersionColumn::Extra, Json(extra));
        }
        if let Some(version_page) = &self.version_page {
            update.set(DBProjectVersionColumn::VersionPage, version_page.value());
        }
        if let Some(publisher) = &self.publisher {
            update.set(DBProjectVersionColumn::Publisher, *publisher);
        }

        update.query().execute(database).await?;

        Ok(())
    }
}

use sqlx::types::Json;
use uuid::Uuid;

use super::{DBProjectVersion, DBProjectVersionColumn};
use crate::{
    database::prelude::*,
    repository::project::{ReleaseType, VersionData},
};

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
            .set(DBProjectVersionColumn::UpdatedAt, SqlFunctionBuilder::now())
            // Without this the builder emits no WHERE clause at all, and the statement rewrites
            // every row in the table. `Id` used to be passed to `set`, which put the target's id
            // in the SET list instead of the filter.
            .filter(DBProjectVersionColumn::Id.equals(version_id.value()));

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

    /// Builds the same statement [UpdateProjectVersion::update] runs, so it can be inspected
    /// without a database.
    #[cfg(test)]
    fn build_sql(&self, version_id: Uuid) -> String {
        let mut update = UpdateQueryBuilder::new(DBProjectVersion::table_name());
        update
            .set(DBProjectVersionColumn::UpdatedAt, SqlFunctionBuilder::now())
            .filter(DBProjectVersionColumn::Id.equals(version_id.value()));

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

        update.format_sql_query().to_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repository::project::ReleaseType;

    /// The statement must be scoped to one row.
    ///
    /// This previously passed the target id to `set` rather than `filter`, which produced an
    /// `UPDATE ... SET id = $1` with no `WHERE` — it rewrote the primary key of every version in
    /// the table. Maven reached it on any re-deploy of an existing version.
    #[test]
    fn update_is_scoped_to_one_row() {
        let sql = UpdateProjectVersion::default().build_sql(Uuid::new_v4());

        let (set_clause, where_clause) = sql
            .split_once(" WHERE ")
            .unwrap_or_else(|| panic!("no WHERE clause — this rewrites every row: {sql}"));

        assert!(
            !set_clause.contains("id ="),
            "statement assigns the primary key instead of filtering on it: {sql}"
        );
        assert!(
            where_clause.contains("id ="),
            "statement is not filtered by id: {sql}"
        );
    }

    /// The filter must survive however many optional fields are set.
    #[test]
    fn update_is_scoped_with_every_field_set() {
        let update = UpdateProjectVersion {
            release_type: Some(ReleaseType::Stable),
            publisher: Some(Some(1)),
            version_page: Some(Some("page".to_owned())),
            extra: Some(VersionData::default()),
        };
        let sql = update.build_sql(Uuid::new_v4());

        assert!(
            sql.contains("WHERE"),
            "statement has no WHERE clause: {sql}"
        );
    }
}

use serde::Serialize;
use sqlx::{FromRow, PgPool, postgres::PgRow};
use tracing::instrument;
use utoipa::ToSchema;
use uuid::Uuid;
use versions::{DBProjectVersion, DBProjectVersionColumn, ProjectVersionType};
mod new;
pub mod utils;
use crate::{database::prelude::*, repository::project::ReleaseType};
pub use new::*;
pub mod info;
pub mod members;
pub mod update;
pub mod versions;
/// Implemented on different types of Project query result. Such as ProjectLookupResult
pub trait ProjectDBType: for<'r> FromRow<'r, PgRow> + Unpin + Send + Sync + TableQuery {
    fn id(&self) -> Uuid;
    async fn find_by_id(id: Uuid, database: &PgPool) -> Result<Option<Self>, sqlx::Error> {
        let project = SelectQueryBuilder::with_columns(DBProject::table_name(), Self::columns())
            .filter(DBProjectColumn::Id.equals(id.value()))
            .query_as()
            .fetch_optional(database)
            .await?;

        Ok(project)
    }
    #[instrument(skip(database))]
    async fn find_by_project_key(
        project_key: &str,
        repository: Uuid,
        database: &PgPool,
    ) -> Result<Option<Self>, sqlx::Error> {
        let project = SelectQueryBuilder::with_columns(DBProject::table_name(), Self::columns())
            .filter(DBProjectColumn::RepositoryId.equals(repository.value()))
            .filter(
                DBProjectColumn::Key
                    .lower()
                    .equals(project_key.to_lowercase().value()),
            )
            .query_as()
            .fetch_optional(database)
            .await?;

        Ok(project)
    }
    #[instrument(skip(database))]
    async fn find_by_project_directory(
        directory: &str,
        repository: Uuid,
        database: &PgPool,
    ) -> Result<Option<Self>, sqlx::Error> {
        let project = SelectQueryBuilder::with_columns(DBProject::table_name(), Self::columns())
            .filter(DBProjectColumn::RepositoryId.equals(repository.value()))
            .filter(
                DBProjectColumn::Path
                    .lower()
                    .equals(directory.to_lowercase().value()),
            )
            .query_as()
            .fetch_optional(database)
            .await?;

        Ok(project)
    }

    /// Finds a Project by the directory of the version
    async fn find_by_version_directory(
        directory: &str,
        repository: Uuid,
        database: &PgPool,
    ) -> Result<Option<Self>, sqlx::Error> {
        let project = SelectQueryBuilder::with_columns(DBProject::table_name(), Self::columns())
            .filter(DBProjectColumn::RepositoryId.equals(repository.value()))
            .join(DBProjectVersion::table_name(), JoinType::Full, |join| {
                join.on(DBProjectVersionColumn::ProjectId.equals(DBProjectColumn::Id))
            })
            .filter(
                DBProjectVersionColumn::Path
                    .lower()
                    .equals(directory.to_lowercase().value()),
            )
            .query_as()
            .fetch_optional(database)
            .await?;
        Ok(project)
    }
    async fn latest_version<V: ProjectVersionType>(
        project_id: Uuid,
        release_type: ReleaseType,
        database: &PgPool,
    ) -> Result<Option<V>, sqlx::Error> {
        let version: Option<V> = SelectQueryBuilder::with_columns(
            <DBProjectVersion as TableType>::table_name(),
            V::columns(),
        )
        .filter(
            DBProjectVersionColumn::ProjectId
                .equals(project_id.value())
                .and(DBProjectVersionColumn::ReleaseType.equals(release_type.value())),
        )
        .order_by(DBProjectVersionColumn::CreatedAt, SQLOrder::Descending)
        .limit(1)
        .query_as()
        .fetch_optional(database)
        .await?;

        Ok(version)
    }
    async fn find_version_by_release_type<V: ProjectVersionType>(
        &self,
        release_types: Vec<ReleaseType>,
        database: &PgPool,
    ) -> DBResult<Vec<V>> {
        let versions: Vec<V> = SelectQueryBuilder::with_columns(
            <DBProjectVersion as TableType>::table_name(),
            V::columns(),
        )
        .filter(DBProjectVersionColumn::ProjectId.equals(self.id().value()))
        .filter(DBProjectVersionColumn::ReleaseType.equals(release_types.value().any()))
        .order_by(DBProjectVersionColumn::CreatedAt, SQLOrder::Descending)
        .query_as()
        .fetch_all(database)
        .await?;
        Ok(versions)
    }
    async fn find_latest_version(&self, database: &PgPool) -> DBResult<Option<DBProjectVersion>> {
        let version: Option<DBProjectVersion> = SelectQueryBuilder::with_columns(
            DBProjectVersion::table_name(),
            DBProjectVersion::columns(),
        )
        .filter(DBProjectVersionColumn::ProjectId.equals(self.id().value()))
        .order_by(DBProjectVersionColumn::CreatedAt, SQLOrder::Descending)
        .limit(1)
        .query_as()
        .fetch_optional(database)
        .await?;
        Ok(version)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, FromRow, ToSchema, TableType)]
#[table(name = "projects")]
pub struct DBProject {
    pub id: Uuid,
    /// Maven will use the groupId
    /// Cargo will be None
    /// NPM will use scope if it's set
    pub scope: Option<String>,
    /// Maven will use something like `{groupId}:{artifactId}`
    /// Cargo will use the `name` field
    ///
    /// This field is unique per repository
    pub key: String,
    /// Name of the project
    ///
    /// Maven will use the artifactId
    /// Cargo will use the `name` field
    /// NPM will use the `name` field
    pub name: String,
    /// A short description of the project
    pub description: Option<String>,
    /// The repository it belongs to
    pub repository_id: Uuid,
    /// The path to the project in the repository
    pub path: String,
    /// Last time the project was updated. This is updated when a new version is added
    pub updated_at: chrono::DateTime<chrono::FixedOffset>,
    /// When the project was created
    pub created_at: chrono::DateTime<chrono::FixedOffset>,
}

impl ProjectDBType for DBProject {
    fn id(&self) -> Uuid {
        self.id
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, FromRow)]
pub struct ProjectIds {
    pub project_id: Uuid,
    pub version_id: Uuid,
}

pub async fn latest_version(
    project_id: Uuid,
    release_type: ReleaseType,
    database: &PgPool,
) -> Result<Option<i32>, sqlx::Error> {
    let version_id: Option<i32> = SelectQueryBuilder::with_columns(
        DBProjectVersion::table_name(),
        vec![DBProjectVersionColumn::Id],
    )
    .filter(
        DBProjectVersionColumn::ProjectId
            .equals(project_id.value())
            .and(DBProjectVersionColumn::ReleaseType.equals(release_type.value())),
    )
    .order_by(DBProjectVersionColumn::CreatedAt, SQLOrder::Descending)
    .limit(1)
    .query_scalar()
    .fetch_optional(database)
    .await?;
    Ok(version_id)
}

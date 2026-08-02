use serde::Serialize;
use sqlx::{FromRow, PgPool, postgres::PgRow};
use tracing::instrument;
use utoipa::ToSchema;
use uuid::Uuid;
use versions::{DBProjectVersion, DBProjectVersionColumn, ProjectVersionType, VersionName};
mod new;
pub mod utils;
pub use new::*;

use crate::{database::prelude::*, repository::project::ReleaseType};
pub mod dist_tags;
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

impl DBProject {
    /// Removes a project and, by cascade, its versions, members and dist-tags.
    ///
    /// Needed by `npm unpublish` when the last version of a package goes away — npm removes the
    /// package entirely at that point rather than leaving an empty packument behind.
    #[instrument(skip(database))]
    pub async fn delete_by_id(id: Uuid, database: &PgPool) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM projects WHERE id = $1")
            .bind(id)
            .execute(database)
            .await?;
        Ok(())
    }
}

/// Every release type that is not a stable release.
///
/// Kept as one list so "latest pre-release" means the same thing everywhere.
pub const PRE_RELEASE_TYPES: [ReleaseType; 4] = [
    ReleaseType::ReleaseCandidate,
    ReleaseType::Beta,
    ReleaseType::Alpha,
    ReleaseType::Snapshot,
];

/// A project as the web API returns it.
///
/// `DBProject` used to be serialized straight onto the wire, which named the key `key` and the path
/// `path` and carried no version at all — while every consumer reads `project_key`, `storage_path`,
/// `latest_release` and `latest_pre_release`. All four came back missing, so a crate's page showed
/// `undefined`, no version was ever found, and the Gradle snippet read `undefined:latest`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, ToSchema)]
pub struct ProjectResponse {
    pub id: Uuid,
    /// Maven's groupId, npm's scope, or nothing for a registry without scopes.
    pub scope: Option<String>,
    /// The project's unique-per-repository key. Maven's `{groupId}:{artifactId}`, a crate name, an
    /// npm package name, a docker image name.
    pub project_key: String,
    /// Maven's artifactId, or the package/crate/image name.
    pub name: String,
    pub description: Option<String>,
    pub repository_id: Uuid,
    /// Where the project lives in the repository, for linking straight into browse.
    pub storage_path: String,
    /// Newest version whose release type is `Stable`.
    pub latest_release: Option<String>,
    /// Newest version that is not a stable release.
    pub latest_pre_release: Option<String>,
    pub updated_at: chrono::DateTime<chrono::FixedOffset>,
    pub created_at: chrono::DateTime<chrono::FixedOffset>,
}

impl ProjectResponse {
    #[instrument(skip(database), name = "Project Response")]
    pub async fn from_project(project: DBProject, database: &PgPool) -> DBResult<Self> {
        let latest_release =
            DBProject::latest_version::<VersionName>(project.id, ReleaseType::Stable, database)
                .await?
                .map(|version| version.version);
        // The query orders newest-first, so the first row is the newest pre-release.
        let latest_pre_release = project
            .find_version_by_release_type::<VersionName>(PRE_RELEASE_TYPES.to_vec(), database)
            .await?
            .into_iter()
            .next()
            .map(|version| version.version);

        let DBProject {
            id,
            scope,
            key,
            name,
            description,
            repository_id,
            path,
            updated_at,
            created_at,
        } = project;

        Ok(Self {
            id,
            scope,
            project_key: key,
            name,
            description,
            repository_id,
            // Maven stores a trailing slash on the project directory and Cargo does not. The path
            // is handed straight to the browse route, so it is normalized here rather than leaving
            // every consumer to cope with both shapes.
            storage_path: path.trim_end_matches('/').to_owned(),
            latest_release,
            latest_pre_release,
            updated_at,
            created_at,
        })
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

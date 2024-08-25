use derive_builder::Builder;
use http::version;
use sqlx::{types::Json, Execute, PgPool, QueryBuilder};
use tracing::{info, instrument, warn};

use crate::repository::project::{ReleaseType, VersionData};

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct UpdateProjectVersion {
    pub release_type: Option<ReleaseType>,
    //#[builder(default)]
    //pub publisher: Option<i32>,
    //#[builder(default)]
    //pub version_page: Option<String>,
    pub extra: Option<VersionData>,
}

impl UpdateProjectVersion {
    #[instrument(name = "UpdateProjectVersion::update")]
    pub async fn update(self, version_id: i32, db: &PgPool) -> Result<(), sqlx::Error> {
        let mut query = QueryBuilder::new("UPDATE project_versions SET updated_at = NOW(), ");
        let mut separated = query.separated(", ");
        if let Some(release_type) = self.release_type {
            separated.push("release_type = ");
            separated.push_bind_unseparated(release_type);
        }
        //if let Some(version_page) = self.version_page {
        //    separated.push("user_manager = ");
        //    separated.push_bind_unseparated(version_page);
        //}
        if let Some(extra) = self.extra {
            separated.push("extra = ");
            separated.push_bind_unseparated(Json(extra));
        }
        query.push(" WHERE id = ");
        query.push_bind(version_id);
        let query: sqlx::query::Query<sqlx::Postgres, sqlx::postgres::PgArguments> = query.build();
        info!(
            "Updating project_version for version_id {} {}",
            version_id,
            query.sql()
        );
        let result = query.execute(db).await?;
        if result.rows_affected() == 0 {
            warn!(
                "No rows affected when updating project_version for version_id {}",
                version_id
            );
        }
        Ok(())
    }
}

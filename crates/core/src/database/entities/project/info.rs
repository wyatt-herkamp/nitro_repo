use pg_extended_sqlx_queries::{
    Aliasable, DynEncodeType, FilterExpr, JoinType, QueryTool, SelectQueryBuilder, TableType,
    WhereableTool,
};
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use uuid::Uuid;

use super::{
    DBProject, DBProjectColumn,
    versions::{DBProjectVersion, DBProjectVersionColumn},
};
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct ProjectInfo {
    pub project_id: Uuid,
    pub project_key: String,
    pub project_scope: String,
    pub project_name: String,
    #[sqlx(default)]
    pub project_version: Option<String>,
    #[sqlx(default)]
    pub version_id: Option<Uuid>,
}
impl ProjectInfo {
    pub async fn query_from_version_id(
        version_id: Uuid,
        database: &sqlx::PgPool,
    ) -> Result<Option<Self>, sqlx::Error> {
        let result = SelectQueryBuilder::new(DBProjectVersion::table_name())
            .select(DBProjectColumn::Key.alias("project_key"))
            .select(DBProjectColumn::Scope.alias("project_scope"))
            .select(DBProjectColumn::Name.alias("project_name"))
            .select(DBProjectVersionColumn::Id.alias("project_id"))
            .select(DBProjectVersionColumn::Version.alias("project_version"))
            .join(DBProject::table_name(), JoinType::Full, |join| {
                join.on(DBProjectVersionColumn::ProjectId.equals(DBProjectColumn::Id))
            })
            .filter(DBProjectVersionColumn::Id.equals(version_id.value()))
            .query_as()
            .fetch_optional(database)
            .await?;
        Ok(result)
    }

    pub async fn query_from_project_id(
        project_id: Uuid,
        database: &sqlx::PgPool,
    ) -> Result<Option<Self>, sqlx::Error> {
        let project_info = SelectQueryBuilder::new(DBProject::table_name())
            .select(DBProjectColumn::Key.alias("project_key"))
            .select(DBProjectColumn::Scope.alias("project_scope"))
            .select(DBProjectColumn::Name.alias("project_name"))
            .select(DBProjectColumn::Id.alias("project_id"))
            .filter(DBProjectColumn::Id.equals(project_id.value()))
            .query_as()
            .fetch_optional(database)
            .await?;

        Ok(project_info)
    }
}

use chrono::{DateTime, FixedOffset};
use pg_extended_sqlx_queries::{
    DynEncodeType, FilterExpr, QueryTool, SQLOrder, SelectQueryBuilder, TableQuery, TableType,
    WhereableTool,
};
use serde::Serialize;
use sqlx::prelude::FromRow;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{
    database::{DBResult, entities::project::versions::DBProjectVersionColumn},
    repository::project::ReleaseType,
};

use super::{DBProjectVersion, ProjectVersionType};
#[derive(Debug, Clone, PartialEq, Eq, Serialize, FromRow, ToSchema)]
pub struct VersionHistoryItem {
    pub id: Uuid,
    pub release_type: ReleaseType,
    pub version: String,
    pub updated_at: DateTime<FixedOffset>,
    pub created_at: DateTime<FixedOffset>,
}
impl TableQuery for VersionHistoryItem {
    type Table = DBProjectVersion;

    fn columns() -> Vec<<Self::Table as pg_extended_sqlx_queries::TableType>::Columns>
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
    pub async fn find_by_project_id(
        project_id: Uuid,
        database: &sqlx::PgPool,
    ) -> DBResult<Vec<Self>> {
        let versions =
            SelectQueryBuilder::with_columns(DBProjectVersion::table_name(), Self::columns())
                .filter(DBProjectVersionColumn::ProjectId.equals(project_id.value()))
                .order_by(DBProjectVersionColumn::UpdatedAt, SQLOrder::Descending)
                .query_as()
                .fetch_all(database)
                .await?;
        Ok(versions)
    }
}

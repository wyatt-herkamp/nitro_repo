use derive_builder::Builder;
use serde::Serialize;
use serde_json::Value;
use sqlx::{types::Json, FromRow};
use utoipa::ToSchema;
use uuid::Uuid;

use super::DateTime;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, FromRow, ToSchema)]
pub struct DBStage {
    pub id: Uuid,
    pub repository: Uuid,
    #[schema(value_type = crate::utils::utopia::AnyType)]
    pub stage_state: Json<Value>,
    pub created_by: i32,
    pub created_at: DateTime,
}

impl DBStage {
    pub async fn get_stage_by_id(
        id: Uuid,
        repository: Uuid,
        database: &sqlx::PgPool,
    ) -> Result<Option<DBStage>, sqlx::Error> {
        let query = "SELECT * FROM stages WHERE id = $1 AND repository = $2".to_string();
        let stage = sqlx::query_as(&query)
            .bind(id)
            .bind(repository)
            .fetch_optional(database)
            .await?;
        Ok(stage)
    }
    pub async fn get_files(
        &self,
        database: &sqlx::PgPool,
    ) -> Result<Vec<DBStageFile>, sqlx::Error> {
        let query = "SELECT * FROM stage_files WHERE stage = $1".to_string();
        let files = sqlx::query_as(&query)
            .bind(self.id)
            .fetch_all(database)
            .await?;
        Ok(files)
    }
    pub async fn delete_stage(&self, database: &sqlx::PgPool) -> Result<(), sqlx::Error> {
        let query = "DELETE FROM stages WHERE id = $1".to_string();
        sqlx::query(&query).bind(self.id).execute(database).await?;
        Ok(())
    }
    pub async fn get_all_stages_for_repository(
        repository: Uuid,
        database: &sqlx::PgPool,
    ) -> Result<Vec<DBStage>, sqlx::Error> {
        let query = "SELECT * FROM stages WHERE repository = $1".to_string();
        let stages = sqlx::query_as(&query)
            .bind(repository)
            .fetch_all(database)
            .await?;
        Ok(stages)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, FromRow, ToSchema)]
pub struct DBStageFile {
    pub id: Uuid,
    pub stage: Uuid,
    pub file_name: String,
    pub created_at: DateTime,
}
#[derive(Debug, Clone, PartialEq, Eq, Builder)]
pub struct NewDBStage {
    pub repository: Uuid,
    pub stage_state: Value,
    pub created_by: i32,
}
impl NewDBStage {
    pub async fn insert(&self, database: &sqlx::PgPool) -> Result<DBStage, sqlx::Error> {
        let query = "INSERT INTO stages (repository, stage_state, created_by) VALUES ($1, $2, $3) RETURNING *".to_string();
        let stage = sqlx::query_as(&query)
            .bind(self.repository)
            .bind(Json(self.stage_state.clone()))
            .bind(self.created_by)
            .fetch_one(database)
            .await?;
        Ok(stage)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Builder)]
pub struct NewDBStageFile {
    pub stage: Uuid,
    pub file_name: String,
}
impl NewDBStageFile {
    pub async fn insert(&self, database: &sqlx::PgPool) -> Result<DBStageFile, sqlx::Error> {
        let query =
            "INSERT INTO stage_files (stage, file_name) VALUES ($1, $2) RETURNING *".to_string();
        let stage_file = sqlx::query_as(&query)
            .bind(self.stage)
            .bind(&self.file_name)
            .fetch_one(database)
            .await?;
        Ok(stage_file)
    }
}

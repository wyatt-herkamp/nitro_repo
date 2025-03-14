use std::{fmt::Debug, path::PathBuf, sync::Arc};

use axum::response::{IntoResponse, Response};
use bytes::Bytes;
use chrono::Duration;
use derive_more::derive::Deref;
use http::StatusCode;
use nr_core::database::entities::stages::{DBStage, NewDBStageFile};
use redb::Result;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::{debug, error, instrument};
use uuid::Uuid;

use crate::app::{NitroRepo, config::get_current_directory};
#[derive(Debug, Error)]
pub enum StagingManagerError {
    #[error("Database Error")]
    DBError(#[from] sqlx::Error),
    #[error("IO Error")]
    IOError(#[from] std::io::Error),
}
impl IntoResponse for StagingManagerError {
    fn into_response(self) -> axum::response::Response {
        error!("{}", self);
        let message = format!("Staging Manager Error {:?}. ", self);
        Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(message.into())
            .unwrap()
    }
}
/// Stages are stored locally before being moved to the storage
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct StagingConfig {
    pub staging_dir: PathBuf,
    #[serde(with = "nr_core::utils::duration_serde::as_seconds")]
    pub time_till_cleanup: Duration,
}
impl Default for StagingConfig {
    fn default() -> Self {
        Self {
            staging_dir: get_current_directory().join("staging"),
            time_till_cleanup: Duration::hours(1),
        }
    }
}
pub struct StagingManagerInner {
    repository: Uuid,
    site: NitroRepo,
}
#[derive(Deref, Clone)]
pub struct StagingManager(Arc<StagingManagerInner>);

impl Debug for StagingManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StagingManager")
            .field("repository_id", &self.repository)
            .finish()
    }
}

impl StagingManager {
    pub async fn get_stage(&self, id: Uuid) -> Result<Option<DBStage>, StagingManagerError> {
        let stage = DBStage::get_stage_by_id(id, self.repository, &self.site.database).await?;
        Ok(stage)
    }
    // This function will assume the stage exists.
    #[instrument]
    pub async fn add_file(
        &self,
        stage_id: Uuid,
        file_name: String,
        file: Bytes,
    ) -> Result<(), StagingManagerError> {
        let staging_dir = self
            .site
            .staging_config
            .staging_dir
            .join(stage_id.to_string());
        if !staging_dir.exists() {
            std::fs::create_dir_all(&staging_dir)?;
        }
        let file_path = staging_dir.join(&file_name);
        std::fs::write(file_path, file)?;

        let new_stage_file = NewDBStageFile {
            stage: stage_id,
            file_name,
        };
        debug!(?new_stage_file, "Adding file to stage");
        let new_file = new_stage_file.insert(&self.site.database).await?;
        debug!(?new_file, "File added to stage");
        Ok(())
    }
}

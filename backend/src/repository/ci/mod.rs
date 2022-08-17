mod error;
use crate::system::permissions::permissions_checker::CanIDo;

use crate::error::internal_error::InternalError;
use crate::repository::handler::{CreateRepository, Repository};
use crate::repository::settings::badge::BadgeSettings;
use crate::repository::settings::frontend::Frontend;
use crate::repository::settings::{RepositoryConfig, RepositoryConfigType, RepositoryType};
use crate::storage::models::Storage;
use crate::utils::get_current_time;
use async_trait::async_trait;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use actix_web::Error;
use actix_web::http::header::HeaderMap;
use bytes::Bytes;
use sea_orm::DatabaseConnection;
use crate::authentication::Authentication;
use crate::repository::ci::error::CIError;
use crate::repository::response::RepoResponse;

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
pub struct CISettings {}

impl RepositoryConfigType for CISettings {
    fn config_name() -> &'static str {
        "ci.json"
    }
}

#[derive(Debug)]
pub struct CIHandler<StorageType: Storage> {
    config: RepositoryConfig,
    storage: Arc<StorageType>,
    badge: BadgeSettings,
    frontend: Frontend,
}

impl<StorageType: Storage> CIHandler<StorageType> {
    pub async fn create(
        config: RepositoryConfig,
        storage: Arc<StorageType>,
    ) -> Result<CIHandler<StorageType>, InternalError> {
        Ok(CIHandler {
            config,
            storage,
            badge: Default::default(),
            frontend: Default::default(),
        })
    }
}

impl<S: Storage> Clone for CIHandler<S> {
    fn clone(&self) -> Self {
        CIHandler {
            config: self.config.clone(),
            storage: self.storage.clone(),
            badge: self.badge.clone(),
            frontend: self.frontend.clone(),
        }
    }
}

crate::repository::settings::define_configs_on_handler!(
    CIHandler<StorageType>,
    badge,
    BadgeSettings,
    frontend,
    Frontend
);

impl<S: Storage> CreateRepository<S> for CIHandler<S> {
    type Config = CISettings;
    type Error = InternalError;

    fn create_repository(
        config: Self::Config,
        name: impl Into<String>,
        storage: Arc<S>,
    ) -> Result<(Self, Self::Config), Self::Error>
        where
            Self: Sized,
    {
        let repository_config = RepositoryConfig::new(name, RepositoryType::CI, storage.as_ref().storage_config().generic_config.id.clone());
        Ok((
            CIHandler {
                config: repository_config,
                storage: storage.clone(),
                badge: Default::default(),
                frontend: Default::default(),
            },
            config,
        ))
    }
}

#[async_trait]
impl<StorageType: Storage> Repository<StorageType> for CIHandler<StorageType> {
    fn get_repository(&self) -> &RepositoryConfig {
        &self.config
    }
    fn get_mut_config(&mut self) -> &mut RepositoryConfig {
        &mut self.config
    }
    fn get_storage(&self) -> &StorageType {
        &self.storage
    }

    async fn handle_put(&self, path: &str, _http: &HeaderMap, conn: &DatabaseConnection, authentication: Authentication, _bytes: Bytes) -> Result<RepoResponse, Error> {
        let caller = crate::helpers::write_check!(authentication, conn, self.config);
        let split: Vec<_> = path.split("/").into_iter().map(|s| s.to_owned()).collect();
        if split.len() < 4 {
            return Err(CIError::BadRequest("Invalid Request `{project}/{job_name}/{build}/{artifact_name}`").into());
        }
        let project = split.get(0).unwrap();
        let job_name = split.get(1).unwrap();
        let build = split.get(2).unwrap();
        let path = split.get(3).unwrap();
        return Err(CIError::BadRequest("Not Implemented").into());
    }
}

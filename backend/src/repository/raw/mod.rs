use crate::authentication::Authentication;
use crate::error::internal_error::InternalError;
use crate::repository::handler::{CreateRepository, Repository, RepositoryType};
use crate::repository::response::RepoResponse;
use crate::repository::settings::{RepositoryConfig, RepositoryConfigType};
use crate::storage::models::Storage;
use crate::system::permissions::permissions_checker::CanIDo;
use actix_web::http::header::HeaderMap;
use actix_web::Error;
use async_trait::async_trait;
use bytes::Bytes;
use schemars::JsonSchema;
use sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
#[derive(Debug)]
pub struct RawHandler<StorageType: Storage> {
    config: RepositoryConfig,
    storage: Arc<StorageType>,
}

impl<StorageType: Storage> RawHandler<StorageType> {
    pub async fn create(
        config: RepositoryConfig,
        storage: Arc<StorageType>,
    ) -> Result<RawHandler<StorageType>, InternalError> {
        Ok(RawHandler { config, storage })
    }
}

impl<S: Storage> Clone for RawHandler<S> {
    fn clone(&self) -> Self {
        RawHandler {
            config: self.config.clone(),
            storage: self.storage.clone(),
        }
    }
}
crate::repository::settings::define_configs_on_handler!(RawHandler<StorageType>);

#[async_trait]
impl<StorageType: Storage> Repository<StorageType> for RawHandler<StorageType> {
    fn get_repository(&self) -> &RepositoryConfig {
        &self.config
    }
    fn get_mut_config(&mut self) -> &mut RepositoryConfig {
        &mut self.config
    }
    fn get_storage(&self) -> &StorageType {
        &self.storage
    }

    async fn handle_get(
        &self,
        path: &str,
        _: &HeaderMap,
        conn: &DatabaseConnection,
        authentication: Authentication,
    ) -> Result<RepoResponse, Error> {
        crate::helpers::read_check!(authentication, conn, self.config);

        let response = self
            .storage
            .get_file_as_response(&self.config, path)
            .await
            .map_err(InternalError::from)?;
        Ok(RepoResponse::FileResponse(response))
    }
    async fn handle_put(
        &self,
        path: &str,
        _: &HeaderMap,
        conn: &DatabaseConnection,
        authentication: Authentication,
        bytes: Bytes,
    ) -> Result<RepoResponse, Error> {
        let _ = crate::helpers::write_check!(authentication, conn, self.config);
        let exists = self
            .storage
            .save_file(&self.config, bytes.as_ref(), path)
            .await
            .map_err(InternalError::from)?;
        // Everything was ok
        Ok(RepoResponse::PUTResponse(
            exists,
            format!(
                "/storages/{}/{}/{}",
                &self.storage.storage_config().generic_config.id,
                &self.config.name,
                path
            ),
        ))
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, JsonSchema)]
pub struct RawSettings {}

impl RepositoryConfigType for RawSettings {
    fn config_name() -> &'static str {
        "raw.json"
    }
}

impl<S: Storage> CreateRepository<S> for RawHandler<S> {
    type Config = RawSettings;
    type Error = InternalError;

    fn create_repository(
        config: Self::Config,
        name: impl Into<String>,
        storage: Arc<S>,
    ) -> Result<(Self, Self::Config), Self::Error>
    where
        Self: Sized,
    {
        let repository_config = RepositoryConfig::new(
            name.into(),
            RepositoryType::Raw,
            storage.storage_config().generic_config.id.clone(),
        );
        Ok((
            RawHandler {
                config: repository_config,
                storage,
            },
            config,
        ))
    }
}

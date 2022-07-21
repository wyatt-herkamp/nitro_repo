use actix_web::http::header::HeaderMap;
use actix_web::http::StatusCode;
use actix_web::web::Bytes;
use async_trait::async_trait;
use log::error;
use sea_orm::DatabaseConnection;
use std::ops::Deref;
use std::sync::{Arc, Weak};
use tokio::sync::RwLockReadGuard;

use crate::authentication::Authentication;
use crate::error::api_error::APIError;
use crate::error::internal_error::InternalError;
use crate::repository::handler::{repository_handler, Repository};
use crate::repository::maven::models::Pom;
use crate::repository::response::RepoResponse;
use crate::repository::settings::{Policy, RepositoryConfig, Visibility};
use crate::storage::file::StorageFileResponse;
use crate::storage::models::Storage;
use crate::system::permissions::options::CanIDo;
use crate::system::user::UserModel;
use hosted::HostedMavenRepository;
use proxy::ProxyMavenRepository;
use staging::StagingRepository;
pub mod error;
pub mod hosted;
pub mod models;
pub mod proxy;
pub mod settings;
pub mod staging;
mod utils;

use actix_web::Error;
use settings::{MavenSettings, MavenType};

repository_handler!(
    MavenHandler,
    Hosted,
    HostedMavenRepository,
    Proxy,
    ProxyMavenRepository,
    Staging,
    StagingRepository
);

impl<S: Storage> MavenHandler<S> {
    pub async fn create(
        repository: RepositoryConfig,
        storage: Arc<S>,
    ) -> Result<MavenHandler<S>, InternalError> {
        let result = repository.get_config::<MavenSettings, S>(&storage).await?;
        if let Some(config) = result {
            match config.repository_type {
                MavenType::Hosted { .. } => Ok(HostedMavenRepository {
                    config: repository,
                    storage,
                }
                .into()),
                MavenType::Proxy { proxies } => Ok(ProxyMavenRepository {
                    config: repository,
                    proxy: proxies,
                    storage,
                }
                .into()),
                MavenType::Staging {
                    stage_to,
                    pre_stage_requirements,
                    parent,
                } => {
                    let staging = StagingRepository {
                        config: repository,
                        stage_to,
                        storage,
                        deploy_requirement: pre_stage_requirements,
                        parent,
                    };
                    Ok(staging.into())
                }
            }
        } else {
            Ok(HostedMavenRepository {
                config: repository,
                storage,
            }
            .into())
        }
    }
}

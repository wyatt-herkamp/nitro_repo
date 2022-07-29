use std::sync::Arc;

use actix_web::http::header::HeaderMap;
use actix_web::web::Bytes;
use actix_web::Error;
use async_trait::async_trait;
use sea_orm::DatabaseConnection;

use hosted::HostedMavenRepository;
use proxy::ProxyMavenRepository;
use settings::{MavenSettings, MavenType};
use staging::StagingRepository;

use crate::authentication::Authentication;
use crate::error::internal_error::InternalError;
use crate::repository::handler::{repository_config_group, repository_handler, Repository};
use crate::repository::maven::proxy::MavenProxySettings;
use crate::repository::maven::settings::ProxySettings;
use crate::repository::maven::staging::{MavenStagingConfig, StageSettings};
use crate::repository::response::RepoResponse;
use crate::repository::settings::badge::BadgeSettings;
use crate::repository::settings::frontend::Frontend;
use crate::repository::settings::RepositoryConfig;
use crate::storage::models::Storage;

pub mod error;
pub mod hosted;
pub mod models;
pub mod proxy;
pub mod settings;
pub mod staging;
mod utils;

repository_handler!(
    MavenHandler,
    Hosted,
    HostedMavenRepository,
    Proxy,
    ProxyMavenRepository,
    Staging,
    StagingRepository
);
use crate::repository::nitro::nitro_repository::NitroRepositoryHandler;
use crate::system::user::database::UserSafeData;
crate::repository::nitro::dynamic::nitro_repo_handler!(MavenHandler, Hosted, HostedMavenRepository);
crate::repository::staging::dynamic::gen_dynamic_stage!(MavenHandler, Staging);

impl<S: Storage> MavenHandler<S> {
    pub async fn create(
        repository: RepositoryConfig,
        storage: Arc<S>,
    ) -> Result<MavenHandler<S>, InternalError> {
        // TODO bring other settings from the configs
        let result = repository.get_config::<MavenSettings, S>(&storage).await?;
        if let Some(config) = result {
            match config.repository_type {
                MavenType::Hosted => Ok(HostedMavenRepository {
                    config: repository,
                    storage,
                    badge: Default::default(),
                    frontend: Default::default(),
                }
                .into()),
                MavenType::Proxy => {
                    let settings = repository
                        .get_config::<MavenProxySettings, S>(&storage)
                        .await?
                        .ok_or(InternalError::Error("Proxy settings not found".to_string()))?;
                    Ok(ProxyMavenRepository {
                        config: repository,
                        proxy: settings,
                        badge: Default::default(),
                        frontend: Default::default(),
                        storage,
                    }
                    .into())
                }
                MavenType::Staging => {
                    let settings = repository
                        .get_config::<MavenStagingConfig, S>(&storage)
                        .await?
                        .ok_or(InternalError::Error("Stage settings not found".to_string()))?;

                    let staging = StagingRepository {
                        config: repository,
                        storage,
                        stage_settings: settings,
                    };
                    Ok(staging.into())
                }
            }
        } else {
            Ok(HostedMavenRepository {
                config: repository,
                storage,
                badge: Default::default(),
                frontend: Default::default(),
            }
            .into())
        }
    }
}

repository_config_group!(MavenHandler, MavenStagingConfig, Staging);
repository_config_group!(MavenHandler, MavenProxySettings, Proxy);

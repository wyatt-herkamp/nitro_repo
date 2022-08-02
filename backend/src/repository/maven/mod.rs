use std::sync::Arc;

use actix_web::http::header::HeaderMap;
use actix_web::http::StatusCode;
use actix_web::web::Bytes;
use actix_web::Error;
use async_trait::async_trait;
use maven_rs::pom::Pom;
use sea_orm::DatabaseConnection;

use hosted::HostedMavenRepository;
use proxy::ProxyMavenRepository;
use settings::{MavenSettings, MavenType};
use staging::StagingRepository;

use crate::authentication::Authentication;
use crate::error::api_error::APIError;
use crate::error::internal_error::InternalError;
use crate::repository::handler::{repository_config_group, repository_handler, Repository};
use crate::repository::maven::proxy::MavenProxySettings;

use crate::repository::maven::staging::MavenStagingConfig;
use crate::repository::response::RepoResponse;

use crate::repository::settings::{Policy, RepositoryConfig};
use crate::storage::models::Storage;

pub mod error;
pub mod hosted;
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
use crate::repository::nitro::VersionData;
use crate::system::user::database::UserSafeData;
use crate::utils::get_current_time;
crate::repository::nitro::dynamic::nitro_repo_handler!(MavenHandler, Hosted, HostedMavenRepository);
crate::repository::staging::dynamic::gen_dynamic_stage!(MavenHandler, Staging);

impl<S: Storage> MavenHandler<S> {
    pub async fn create(
        repository: RepositoryConfig,
        storage: Arc<S>,
    ) -> Result<MavenHandler<S>, InternalError> {
        // TODO bring other settings from the configs
        let config = repository
            .get_config::<MavenSettings, S>(&storage)
            .await?
            .unwrap_or_default();

        match config.repository_type {
            MavenType::Hosted => Ok(HostedMavenRepository {
                config: repository,
                storage,
                badge: Default::default(),
                frontend: Default::default(),
                hosted: Default::default(),
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
    }
}

repository_config_group!(MavenHandler, MavenStagingConfig, Staging);
repository_config_group!(MavenHandler, MavenProxySettings, Proxy);
impl From<Pom> for VersionData {
    fn from(pom: Pom) -> Self {
        VersionData {
            name: format!("{}:{}", &pom.group_id, &pom.artifact_id),
            description: pom.description.unwrap_or_default(),
            source: None,
            licence: None,
            version: pom.version,
            created: get_current_time(),
        }
    }
}

pub fn validate_policy(policy: &Policy, version: impl AsRef<str>) -> Option<RepoResponse> {
    match policy {
        Policy::Release => {
            if version.as_ref().contains("-SNAPSHOT") {
                return Some(
                    APIError::from(("Release in a snapshot only", StatusCode::BAD_REQUEST)).into(),
                );
            }
        }
        Policy::Snapshot => {
            if !version.as_ref().contains("-SNAPSHOT") {
                return Some(
                    APIError::from(("Snapshot in a release only", StatusCode::BAD_REQUEST)).into(),
                );
            }
        }
        _ => {}
    }
    None
}

use std::sync::Arc;

use actix_web::http::header::HeaderMap;

use actix_web::web::Bytes;
use actix_web::Error;
use async_trait::async_trait;
use chrono::Local;
use log::debug;
use maven_rs::pom::Pom;
use sea_orm::DatabaseConnection;

use hosted::HostedMavenRepository;
use proxy::ProxyMavenRepository;
use settings::{MavenSettings, MavenType};
use staging::StagingRepository;

use crate::authentication::Authentication;

use crate::error::internal_error::InternalError;
use crate::repository::handler::{
    repository_handler, CreateRepository, Repository, RepositoryType,
};
use crate::repository::maven::error::MavenError;
use crate::repository::maven::hosted::MavenHosted;
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

repository_handler!(
    MavenHandler,
    Hosted,
    HostedMavenRepository,
    Proxy,
    ProxyMavenRepository,
    Staging,
    StagingRepository
);
use crate::repository::nitro::{ProjectSource, VersionData};
use crate::repository::settings::badge::BadgeSettings;
use crate::repository::settings::frontend::Frontend;
use crate::repository::settings::repository_page::RepositoryPage;
use crate::system::user::database::UserSafeData;
crate::repository::nitro::dynamic::nitro_repo_handler!(
    MavenHandler,
    Hosted,
    HostedMavenRepository,
    Proxy,
    ProxyMavenRepository
);
crate::repository::staging::dynamic::gen_dynamic_stage!(MavenHandler, Staging);

impl<S: Storage> MavenHandler<S> {
    pub async fn create(
        repository: RepositoryConfig,
        storage: Arc<S>,
    ) -> Result<MavenHandler<S>, InternalError> {
        let config = repository
            .get_config::<MavenSettings, S>(&storage)
            .await?
            .unwrap_or_default();

        match config.repository_type {
            MavenType::Hosted => Ok(HostedMavenRepository {
                badge: repository
                    .get_config(storage.as_ref())
                    .await?
                    .unwrap_or_default(),
                frontend: repository
                    .get_config(storage.as_ref())
                    .await?
                    .unwrap_or_default(),
                hosted: repository
                    .get_config(storage.as_ref())
                    .await?
                    .unwrap_or_default(),
                repository_page: repository
                    .get_config(storage.as_ref())
                    .await?
                    .unwrap_or_default(),
                config: repository,
                storage,
            }
            .into()),
            MavenType::Proxy => Ok(ProxyMavenRepository {
                proxy: repository
                    .get_config::<MavenProxySettings, S>(storage.as_ref())
                    .await?
                    .unwrap_or_default(),
                badge: repository
                    .get_config(storage.as_ref())
                    .await?
                    .unwrap_or_default(),
                frontend: repository
                    .get_config(storage.as_ref())
                    .await?
                    .unwrap_or_default(),
                repository_page: repository
                    .get_config(storage.as_ref())
                    .await?
                    .unwrap_or_default(),
                config: repository,

                storage,
            }
            .into()),
            MavenType::Staging => {
                let settings = repository
                    .get_config::<MavenStagingConfig, S>(storage.as_ref())
                    .await?
                    .unwrap_or_default();
                let staging = StagingRepository {
                    repository_page: repository
                        .get_config(storage.as_ref())
                        .await?
                        .unwrap_or_default(),
                    config: repository,
                    storage,
                    stage_settings: settings,
                };
                Ok(staging.into())
            }
        }
    }
}

crate::repository::handler::repository_config_group!(
    MavenHandler,
    RepositoryPage,
    Staging,
    Proxy,
    Hosted
);

impl From<Pom> for VersionData {
    fn from(pom: Pom) -> Self {
        let source = pom.scm.and_then(|v| {
            if let Some(ty) = v.connection {
                if ty.starts_with("scm::git:") {
                    let string = ty.replace("scm::git:", "");
                    Some(ProjectSource::Git {
                        url: string.to_string(),
                    })
                } else {
                    debug!("Unknown scm type: {}", ty);
                    None
                }
            } else {
                None
            }
        });
        VersionData {
            name: format!("{}:{}", &pom.group_id, &pom.artifact_id),
            description: pom.description.unwrap_or_default(),
            source,
            licence: None,
            version: pom.version,
            created: Local::now().into(),
        }
    }
}

pub fn validate_policy(policy: &Policy, version: impl AsRef<str>) -> Result<(), MavenError> {
    match policy {
        Policy::Release => {
            if version.as_ref().contains("-SNAPSHOT") {
                return Err(MavenError::PolicyError(policy.clone()));
            }
        }
        Policy::Snapshot => {
            if !version.as_ref().contains("-SNAPSHOT") {
                return Err(MavenError::PolicyError(policy.clone()));
            }
        }
        _ => {}
    }
    Ok(())
}

impl<S: Storage> CreateRepository<S> for MavenHandler<S> {
    type Config = MavenSettings;
    type Error = InternalError;

    fn create_repository(
        config: MavenSettings,
        name: impl Into<String>,
        storage: Arc<S>,
    ) -> Result<(Self, MavenSettings), Self::Error>
    where
        Self: Sized,
    {
        let repository_config = RepositoryConfig {
            name: name.into(),
            visibility: Default::default(),
            active: true,
            require_token_over_basic: false,
            repository_type: RepositoryType::Maven,
            created: Local::now().into(),
            storage: storage.storage_config().generic_config.id.clone(),
        };
        match config.repository_type {
            MavenType::Hosted => {
                let hosted = HostedMavenRepository {
                    badge: BadgeSettings::default(),
                    frontend: Frontend::default(),
                    hosted: MavenHosted::default(),
                    config: repository_config,
                    storage,
                    repository_page: RepositoryPage::default(),
                };
                Ok((hosted.into(), config))
            }
            MavenType::Staging => {
                let staging = StagingRepository {
                    config: repository_config,
                    storage,
                    stage_settings: MavenStagingConfig::default(),
                    repository_page: RepositoryPage::default(),
                };
                Ok((staging.into(), config))
            }
            MavenType::Proxy => {
                let proxy = ProxyMavenRepository {
                    proxy: MavenProxySettings::default(),
                    badge: BadgeSettings::default(),
                    frontend: Frontend::default(),
                    config: repository_config,
                    storage,
                    repository_page: RepositoryPage::default(),
                };
                Ok((proxy.into(), config))
            }
        }
    }
}

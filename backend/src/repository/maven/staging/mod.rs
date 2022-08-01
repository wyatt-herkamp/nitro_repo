use crate::authentication::Authentication;
use crate::error::api_error::APIError;
use crate::error::internal_error::InternalError;
use crate::repository::handler::Repository;

use crate::repository::maven::settings::ProxySettings;
use crate::repository::response::RepoResponse;
use crate::repository::settings::{Policy, RepositoryConfig, RepositoryConfigType};
use crate::repository::staging::StageHandler;
use crate::storage::file::StorageFileResponse;
use crate::storage::models::Storage;
use crate::storage::multi::MultiStorageController;
use crate::storage::DynamicStorage;
use crate::system::permissions::options::CanIDo;

use actix_web::http::header::HeaderMap;
use actix_web::http::StatusCode;
use actix_web::web::Bytes;
use actix_web::{Error, HttpResponse};
use async_trait::async_trait;

use futures_util::stream::StreamExt;
use futures_util::SinkExt;
use sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};

use crate::system::user::database::UserSafeData;

use log::{error, info};
use std::sync::Arc;

mod external;
mod git;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MavenStagingConfig {
    /// This is a parent that nothing is actually pushed it. It just allows for data retrieval.
    parent: ProxySettings,
    stage_to: Vec<StageSettings>,
    pre_stage_requirements: Vec<DeployRequirement>,
}

impl RepositoryConfigType for MavenStagingConfig {
    fn config_name() -> &'static str {
        "maven_staging.json"
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum StageSettings {
    InternalRepository {
        storage: String,
        repository: String,
    },
    // The Po mans Repository. (For https://github.com/NickAcPT)
    GitPush {
        git_url: String,
        git_branch: String,
        git_username: String,
        git_password: String,
    },
    ExternalRepository {
        repository: String,
        username: String,
        password: String,
    },
}
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum DeployRequirement {}
#[derive(Debug)]
pub struct StagingRepository<S: Storage> {
    pub config: RepositoryConfig,
    pub storage: Arc<S>,
    pub stage_settings: MavenStagingConfig,
}
crate::repository::settings::define_config_handler!(
    stage_settings,
    StagingRepository<StorageType>,
    MavenStagingConfig
);
#[async_trait]
impl<'a, S: Storage> StageHandler<S> for StagingRepository<S> {
    async fn push(
        &self,
        directory: String,
        storages: Arc<MultiStorageController<DynamicStorage>>,
        model: UserSafeData,
    ) -> Result<(), InternalError> {
        for stage in self.stage_settings.stage_to.iter() {
            match stage {
                StageSettings::InternalRepository { .. } => {}
                StageSettings::GitPush {
                    git_branch,
                    git_url,
                    git_password,
                    git_username,
                } => {
                    // Clone all data
                    let branch = git_branch.clone();
                    let url = git_url.clone();
                    let password = git_password.clone();
                    let username = git_username.clone();
                    let repository = self.get_repository().clone();
                    let storage = storages
                        .get_storage_by_name(&self.config.storage)
                        .await
                        .unwrap()
                        .unwrap();
                    let directory = directory.clone();
                    let user = model.clone();
                    actix_web::rt::spawn(async move {
                        info!("Triggering Git Stage");
                        if let Err(error) = git::stage_to_git(
                            username, password, url, branch, directory, storage, repository, user,
                        )
                        .await
                        {
                            error!("Failed to stage to git. The rest of the stages do continue to matter. ");
                            error!("{}", error);
                            // Trigger Webhooks
                        }
                    });
                }
                StageSettings::ExternalRepository {
                    username,
                    repository,
                    password,
                } => {
                    let url = repository.clone();
                    let _repository = repository.clone();
                    let username = username.clone();
                    let password = password.clone();
                    let repository = self.get_repository().clone();
                    let storage = storages
                        .get_storage_by_name(&self.config.storage)
                        .await
                        .unwrap()
                        .unwrap();
                    let directory = directory.clone();
                    let user = model.clone();

                    actix_web::rt::spawn(async move {
                        info!("Triggering External Stage");
                        if let Err(error) = external::stage_to_external(
                            username, password, url, directory, storage, repository, user,
                        )
                        .await
                        {
                            error!("Failed to stage to external. The rest of the stages do continue to matter. ");
                            error!("{}", error);
                            // Trigger Webhooks
                        }
                    });
                }
            }
        }
        Ok(())
    }
}

impl<S: Storage> Clone for StagingRepository<S> {
    fn clone(&self) -> Self {
        StagingRepository {
            config: self.config.clone(),
            storage: self.storage.clone(),
            stage_settings: self.stage_settings.clone(),
        }
    }
}

#[async_trait]
impl<S: Storage> Repository<S> for StagingRepository<S> {
    fn get_repository(&self) -> &RepositoryConfig {
        &self.config
    }
    fn get_mut_config(&mut self) -> &mut RepositoryConfig {
        &mut self.config
    }
    fn get_storage(&self) -> &S {
        self.storage.as_ref()
    }
    fn features(&self) -> Vec<&'static str> {
        vec!["proxy"]
    }
    async fn handle_get(
        &self,
        path: &str,
        _http: &HeaderMap,
        conn: &DatabaseConnection,
        authentication: Authentication,
    ) -> Result<RepoResponse, Error> {
        crate::helpers::read_check!(authentication, conn, self.config);

        match self
            .storage
            .get_file_as_response(&self.config, path)
            .await
            .map_err(InternalError::from)?
        {
            StorageFileResponse::List(list) => Ok(RepoResponse::try_from((list, StatusCode::OK))?),

            StorageFileResponse::NotFound => {
                let builder = reqwest::ClientBuilder::new()
                    .user_agent("Nitro Repo Staging Service")
                    .build()
                    .unwrap();
                let url = format!("{}/{}", self.stage_settings.parent.proxy_url, path);
                let response = builder.get(&url).send().await;
                if let Ok(response) = response {
                    if response.status().is_success() {
                        let stream = response.bytes_stream();
                        return Ok(RepoResponse::HttpResponse(
                            HttpResponse::Ok().streaming(stream),
                        ));
                    }
                }

                Ok(RepoResponse::FileResponse(StorageFileResponse::NotFound))
            }
            v => Ok(RepoResponse::FileResponse(v)),
        }
    }

    async fn handle_put(
        &self,
        path: &str,
        _: &HeaderMap,
        conn: &DatabaseConnection,
        authentication: Authentication,
        bytes: Bytes,
    ) -> Result<RepoResponse, actix_web::Error> {
        crate::helpers::write_check!(authentication, conn, self.config);
        match self.config.policy {
            Policy::Release => {
                if path.contains("-SNAPSHOT") {
                    return Err(APIError::from((
                        "SNAPSHOT in release only",
                        StatusCode::BAD_REQUEST,
                    ))
                    .into());
                }
            }
            Policy::Snapshot => {
                if !path.contains("-SNAPSHOT") {
                    return Err(APIError::from((
                        "Release in a snapshot only",
                        StatusCode::BAD_REQUEST,
                    ))
                    .into());
                }
            }
            Policy::Mixed => {}
        }
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

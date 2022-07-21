use crate::authentication::Authentication;
use crate::error::api_error::APIError;
use crate::error::internal_error::InternalError;
use crate::repository::handler::Repository;
use crate::repository::maven::models::Pom;
use crate::repository::maven::settings::{MavenSettings, MavenType, ProxySettings};
use crate::repository::response::RepoResponse;
use crate::repository::settings::{Policy, RepositoryConfig, Visibility};
use crate::repository::staging::{ProcessingStage, ProjectsToStage, StageHandler};
use crate::storage::file::StorageFileResponse;
use crate::storage::models::Storage;
use crate::storage::multi::MultiStorageController;
use crate::storage::DynamicStorage;
use crate::system::permissions::options::CanIDo;
use crate::system::user::UserModel;
use actix_web::http::header::HeaderMap;
use actix_web::http::StatusCode;
use actix_web::web::Bytes;
use actix_web::{Error, HttpResponse};
use async_trait::async_trait;
use bytes::{BufMut, BytesMut};
use futures::channel::mpsc::unbounded;
use futures_util::stream::StreamExt;
use futures_util::SinkExt;
use sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};
use std::ops::Deref;
use std::path::PathBuf;
use std::sync::{Arc, Weak};
use tokio::io::{duplex, AsyncWriteExt};
use tokio::sync::RwLockReadGuard;

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
pub struct StagingRepository<S: Storage> {
    pub config: RepositoryConfig,
    pub storage: Arc<S>,
    pub stage_to: Vec<StageSettings>,
    pub parent: ProxySettings,
    pub deploy_requirement: Vec<DeployRequirement>,
}

impl<'a, S: Storage> StagingRepository<S> {
    pub async fn create(
        repository: RepositoryConfig,
        storage: Arc<S>,
    ) -> Result<StagingRepository<S>, InternalError> {
        let result = repository.get_config::<MavenSettings, S>(&storage).await?;
        if let Some(config) = result {
            match config.repository_type {
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
                _ => {
                    panic!("Staging Repository can only be used with Staging Repository Type");
                }
            }
        } else {
            panic!("Staging Repository can only be used with Staging Repository Type");
        }
    }
}

#[async_trait]
impl<'a, S: Storage> StageHandler<S> for StagingRepository<S> {
    async fn push(
        &self,
        directory: String,
        process: ProcessingStage,
        storages: Arc<MultiStorageController<DynamicStorage>>,
    ) -> Result<(), InternalError> {
        todo!()
    }
}
#[async_trait]
impl<S: Storage> Repository<S> for StagingRepository<S> {
    fn get_repository(&self) -> &RepositoryConfig {
        &self.config
    }

    fn get_storage(&self) -> &S {
        self.storage.as_ref()
    }

    async fn handle_get(
        &self,
        path: &str,
        http: &HeaderMap,
        conn: &DatabaseConnection,
        authentication: Authentication,
    ) -> Result<RepoResponse, Error> {
        if self.config.visibility == Visibility::Private {
            let caller: UserModel = authentication.get_user(conn).await??;
            if let Some(value) = caller.can_read_from(&self.config)? {
                return Err(value.into());
            }
        }

        match self
            .storage
            .get_file_as_response(&self.config, path)
            .await
            .map_err(InternalError::from)?
        {
            StorageFileResponse::List(list) => {
                /*
                let files = self.process_storage_files(list, path).await?;
                Ok(RepoResponse::try_from((files, StatusCode::OK))?)*/
                panic!("Not implemented")
            }

            StorageFileResponse::NotFound => {
                let builder = reqwest::ClientBuilder::new()
                    .user_agent("Nitro Repo Staging Service")
                    .build()
                    .unwrap();
                let url = format!("{}/{}", self.parent.proxy_url, path);
                let mut response = builder.get(&url).send().await;
                if let Ok(mut response) = response {
                    if response.status().is_success() {
                        let mut stream = response.bytes_stream();
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
        let caller: UserModel = authentication.get_user(conn).await??;
        if let Some(_value) = caller.can_deploy_to(&self.config)? {}
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

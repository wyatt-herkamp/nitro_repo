use crate::authentication::Authentication;
use crate::error::api_error::APIError;
use crate::error::internal_error::InternalError;
use crate::repository::handler::Repository;

use crate::repository::response::RepoResponse;
use crate::repository::settings::{Policy, RepositoryConfig, RepositoryConfigType, Visibility};
use crate::storage::file::StorageFileResponse;
use crate::storage::models::Storage;
use crate::system::permissions::options::CanIDo;
use crate::system::user::UserModel;
use actix_web::http::header::HeaderMap;
use actix_web::http::StatusCode;
use actix_web::web::Bytes;
use async_trait::async_trait;

use crate::repository::settings::badge::BadgeSettings;
use crate::repository::settings::frontend::Frontend;
use sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug)]
pub struct HostedMavenRepository<S: Storage> {
    pub config: RepositoryConfig,
    pub storage: Arc<S>,
    pub badge: BadgeSettings,
    pub frontend: Frontend,
}
crate::repository::settings::define_config_handler!(
    badge,
    HostedMavenRepository<StorageType>,
    BadgeSettings
);
crate::repository::settings::define_config_handler!(
    frontend,
    HostedMavenRepository<StorageType>,
    Frontend
);

impl<S: Storage> Clone for HostedMavenRepository<S> {
    fn clone(&self) -> Self {
        HostedMavenRepository {
            config: self.config.clone(),
            storage: self.storage.clone(),
            badge: self.badge.clone(),
            frontend: self.frontend.clone(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MavenHosted {
    pub allow_pushing: bool,
}
impl RepositoryConfigType for MavenHosted {
    fn config_name() -> &'static str {
        "maven_hosted.json"
    }
}

impl Default for MavenHosted {
    fn default() -> Self {
        MavenHosted {
            allow_pushing: true,
        }
    }
}

#[async_trait]
impl<S: Storage> Repository<S> for HostedMavenRepository<S> {
    fn get_repository(&self) -> &RepositoryConfig {
        &self.config
    }

    fn get_mut_config(&mut self) -> &mut RepositoryConfig {
        &mut self.config
    }

    fn get_storage(&self) -> &S {
        self.storage.as_ref()
    }

    async fn handle_get(
        &self,
        path: &str,
        _: &HeaderMap,
        conn: &DatabaseConnection,
        authentication: Authentication,
    ) -> Result<RepoResponse, actix_web::Error> {
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
            StorageFileResponse::List(_list) => {
                /*                let files = self.process_storage_files(list, path).await?;
                Ok(RepoResponse::try_from((files, StatusCode::OK))?)*/
                panic!("Not implemented")
            }
            value => Ok(RepoResponse::FileResponse(value)),
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

        //  Post Deploy Handler
        /*        if path.ends_with(".pom") {
            let vec = bytes.as_ref().to_vec();
            let result = String::from_utf8(vec).map_err(APIError::bad_request)?;
            let pom: Pom = serde_xml_rs::from_str(&result).map_err(APIError::bad_request)?;

            let project_folder = format!("{}/{}", pom.group_id.replace('.', "/"), pom.artifact_id);
            let version_folder = format!("{}/{}", &project_folder, &pom.version);
            if let Err(error) = self
                .post_deploy(project_folder, version_folder, caller, pom.into())
                .await
            {
                error!("Unable to complete post processing Tasks {}", error);
            }
        }*/
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

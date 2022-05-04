use actix_web::http::header::HeaderMap;
use actix_web::http::StatusCode;
use actix_web::web::Bytes;
use std::sync::Arc;

use log::error;
use sea_orm::DatabaseConnection;

use crate::repository::maven::models::{MavenSettings, Pom};
use crate::repository::settings::security::Visibility;
use crate::repository::settings::Policy;

use crate::authentication::Authentication;
use crate::repository::data::{RepositoryConfig, RepositoryType};
use crate::repository::error::RepositoryError;
use crate::repository::handler::RepositoryHandler;
use crate::repository::nitro::nitro_repository::NitroRepositoryHandler;

use crate::repository::response::RepoResponse;
use crate::repository::response::RepoResponse::{BadRequest, NotFound};
use crate::storage::models::Storage;
use crate::system::permissions::options::CanIDo;
use crate::system::user::UserModel;
use async_trait::async_trait;
use tokio::sync::RwLockReadGuard;

pub mod error;
pub mod models;
mod utils;

pub struct MavenHandler<'a> {
    config: RepositoryConfig,
    storage: RwLockReadGuard<'a, Box<dyn Storage>>,
}

impl<'a> MavenHandler<'a> {
    pub fn create(
        repository: RepositoryConfig,
        storage: RwLockReadGuard<'a, Box<dyn Storage>>,
    ) -> Result<MavenHandler<'a>, RepositoryError> {
        Ok(MavenHandler::<'a> {
            config: repository,
            storage,
        })
    }
}
#[async_trait]
impl<'a> RepositoryHandler<'a> for MavenHandler<'a> {
    async fn handle_get(
        &self,
        path: &str,
        _: &HeaderMap,
        conn: &DatabaseConnection,
        authentication: Authentication,
    ) -> Result<RepoResponse, RepositoryError> {
        if self.config.visibility == Visibility::Private {
            let caller: UserModel = authentication.get_user(conn).await??;
            if let Some(value) = caller.can_read_from(&self.config)? {
                return Err(RepositoryError::RequestError(
                    value.to_string(),
                    StatusCode::UNAUTHORIZED,
                ));
            }
        }

        let result = self
            .storage
            .get_file_as_response(&self.config, path)
            .await?;
        if let Some(_result) = result {
            todo!("Unhandled Result Type")
        } else {
            Ok(NotFound)
        }
    }

    async fn handle_put(
        &self,
        path: &str,
        _: &HeaderMap,
        conn: &DatabaseConnection,
        authentication: Authentication,
        bytes: Bytes,
    ) -> Result<RepoResponse, RepositoryError> {
        let caller: UserModel = authentication.get_user(conn).await??;
        if let Some(value) = caller.can_deploy_to(&self.config)? {
            return Err(RepositoryError::RequestError(
                value.to_string(),
                StatusCode::UNAUTHORIZED,
            ));
        } //TODO find a better way to do this
        match self.config.policy {
            Policy::Release => {
                if path.contains("-SNAPSHOT") {
                    return Ok(BadRequest("SNAPSHOT in release only".to_string()));
                }
            }
            Policy::Snapshot => {
                if !path.contains("-SNAPSHOT") {
                    return Ok(BadRequest("Release in a snapshot only".to_string()));
                }
            }
            Policy::Mixed => {}
        }
        self.storage
            .save_file(&self.config, bytes.as_ref(), path)
            .await?;

        //  Post Deploy Handler
        if path.ends_with(".pom") {
            let vec = bytes.as_ref().to_vec();
            let result = String::from_utf8(vec)?;
            let pom: Result<Pom, serde_xml_rs::Error> = serde_xml_rs::from_str(&result);
            if let Err(error) = &pom {
                error!("Unable to Parse Pom File {} Error {}", &path, error);
            }
            if let Ok(pom) = pom {
                let project_folder =
                    format!("{}/{}", pom.group_id.replace('.', "/"), pom.artifact_id);
                let version_folder = format!("{}/{}", &project_folder, &pom.version);
                MavenHandler::post_deploy(
                    &self.storage,
                    &self.config,
                    project_folder,
                    version_folder,
                    caller,
                    pom.into(),
                )
                .await;
            }
        }
        // Everything was ok
        Ok(RepoResponse::Ok)
    }
}

impl NitroRepositoryHandler for MavenHandler<'_> {
    fn parse_project_to_directory<S: Into<String>>(value: S) -> String {
        value.into().replace('.', "/").replace(':', "/")
    }
}

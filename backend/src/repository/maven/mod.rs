use crate::constants::PROJECT_FILE;
use actix_web::http::header::HeaderMap;
use actix_web::http::StatusCode;
use actix_web::web::Bytes;

use log::Level::Trace;
use log::{debug, error, log_enabled, trace};
use sea_orm::DatabaseConnection;

use crate::repository::deploy::{handle_post_deploy, DeployInfo};
use crate::repository::maven::models::{MavenSettings, Pom};
use crate::repository::settings::security::Visibility;
use crate::repository::settings::Policy;

use crate::authentication::Authentication;
use crate::repository::data::{RepositoryConfig, RepositoryMainConfig};
use crate::repository::error::RepositoryError;
use crate::repository::handler::RepositoryHandler;
use crate::repository::nitro::nitro_repository::NitroRepository;
use crate::repository::nitro::utils::update_project_in_repositories;
use crate::repository::response::RepoResponse;
use crate::repository::response::RepoResponse::{BadRequest, NotFound};
use crate::storage::models::Storage;
use crate::system::permissions::options::CanIDo;
use crate::system::user::UserModel;
use async_trait::async_trait;

pub mod error;
pub mod models;
mod utils;

pub struct MavenHandler;

#[async_trait]
impl RepositoryHandler<MavenSettings> for MavenHandler {
    async fn handle_get(
        repository: &RepositoryConfig<MavenSettings>,
        storage: &Storage,
        path: &str,
        _: &HeaderMap,
        conn: &DatabaseConnection,
        authentication: Authentication,
    ) -> Result<RepoResponse, RepositoryError> {
        if repository.main_config.security.visibility == Visibility::Private {
            let caller: UserModel = authentication.get_user(conn).await??;
            if let Some(value) = caller.can_read_from(repository)? {
                return Err(RepositoryError::RequestError(
                    value.to_string(),
                    StatusCode::UNAUTHORIZED,
                ));
            }
        }

        let result = storage.get_file_as_response(repository, path).await?;
        if let Some(_result) = result {
            todo!("Unhandled Result Type")
        } else {
            Ok(NotFound)
        }
    }

    async fn handle_put(
        repository: &RepositoryConfig<MavenSettings>,
        storage: &Storage,
        path: &str,
        _: &HeaderMap,
        conn: &DatabaseConnection,
        authentication: Authentication,
        bytes: Bytes,
    ) -> Result<RepoResponse, RepositoryError> {
        let caller: UserModel = authentication.get_user(conn).await??;
        if let Some(value) = caller.can_deploy_to(repository)? {
            return Err(RepositoryError::RequestError(
                value.to_string(),
                StatusCode::UNAUTHORIZED,
            ));
        } //TODO find a better way to do this
        match repository.main_config.policy {
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
        storage.save_file(&repository, bytes.as_ref(), path).await?;

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
                let repository = repository.clone();
                let storage = storage.clone();
                actix_web::rt::spawn(async move {
                    let storage = storage;
                    let repository = repository;
                    MavenHandler::post_deploy(
                        &storage,
                        &repository,
                        project_folder,
                        version_folder,
                        caller,
                        pom.into(),
                    )
                    .await;
                });
            }
        }
        // Everything was ok
        Ok(RepoResponse::Ok)
    }
}

impl NitroRepository<MavenSettings> for MavenHandler {
    fn parse_project_to_directory<S: Into<String>>(value: S) -> String {
        value.into().replace('.', "/").replace(':', "/")
    }
}

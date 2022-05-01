use crate::constants::PROJECT_FILE;
use actix_web::web::Bytes;
use actix_web::HttpRequest;
use log::Level::Trace;
use log::{debug, error, log_enabled, trace};
use sea_orm::DatabaseConnection;

use crate::error::internal_error::InternalError;
use crate::repository::deploy::{handle_post_deploy, DeployInfo};
use crate::repository::maven::models::{MavenSettings, Pom};
use crate::repository::settings::security::Visibility;
use crate::repository::settings::Policy;

use crate::authentication::Authentication;
use crate::repository::data::RepositoryConfig;
use crate::repository::handler::RepositoryHandler;
use crate::repository::nitro::NitroRepository;
use crate::repository::response::RepoResponse::{BadRequest, NotFound, ProjectResponse};
use crate::repository::response::{Project, RepoResponse};
use crate::repository::utils::{
    get_project_data, get_version_data, get_versions, process_storage_files,
};
use crate::storage::models::Storage;
use crate::system::permissions::options::CanIDo;
use crate::system::user::UserModel;
use async_trait::async_trait;
use crate::repository::error::RepositoryError;

pub mod models;
mod utils;
pub mod error;

pub struct MavenHandler;
#[async_trait]
impl RepositoryHandler<MavenSettings> for MavenHandler {
    async fn handle_get(
        repository: &RepositoryConfig<MavenSettings>,
        storage: &Storage,
        path: &str,
        http: HttpRequest,
        conn: &DatabaseConnection, authentication: Authentication,
    ) -> Result<RepoResponse, RepositoryError> {
        if repository.security.visibility == Visibility::Private {
            let caller: UserModel = authentication.get_user(conn).await??;
            caller.can_read_from(&repository)?;
        }

        let result =
            storage
                .get_file_as_response(repository, &path, &http)
                .await?;
        if let Some(result) = result {
            if result.is_left() {
                Ok(RepoResponse::FileResponse(result.left().unwrap()))
            } else {
                let vec = result.right().unwrap();
                let file_response = process_storage_files(
                    &storage,
                    &repository,
                    vec,
                    &path,
                )
                    .await?;
                Ok(RepoResponse::NitroFileList(file_response))
            }
        } else {
            Ok(NotFound)
        }
    }

    async fn handle_put(
        repository: &RepositoryConfig<MavenSettings>,
        storage: &Storage,
        path: &str,
        http: HttpRequest,
        conn: &DatabaseConnection, authentication: Authentication, bytes: Bytes,
    ) -> Result<RepoResponse, RepositoryError> {
        let caller: UserModel = authentication.get_user(conn).await??;
        caller.can_deploy_to(&repository)?;
        //TODO find a better way to do this
        match repository.settings.policy {
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
            storage
            .save_file(&repository, bytes.as_ref(), &path)
            .await?;
        if path.ends_with(".pom") {
            let vec = bytes.as_ref().to_vec();
            let result = String::from_utf8(vec)?;
            let pom: Result<Pom, serde_xml_rs::Error> = serde_xml_rs::from_str(&result);
            if let Err(error) = &pom {
                error!(
                    "Unable to Parse Pom File {} Error {}",
                    &path, error
                );
            }
            if let Ok(pom) = pom {
                let project_folder =
                    format!("{}/{}", pom.group_id.replace('.', "/"), pom.artifact_id);
                trace!("Project Folder Location {}", project_folder);
                let repository = repository.clone();
                let storage = storage.clone();
                actix_web::rt::spawn(async move {
                    if let Err(error) = utils::update_project(
                        &storage,
                        &repository.init_values,
                        &project_folder,
                        pom.version.clone(),
                        pom.clone(),
                    )
                        .await
                    {
                        error!("Unable to update {}, {}", PROJECT_FILE, error);
                        trace!(
                            "Version {} Name: {}",
                            &pom.version,
                            format!("{}:{}", &pom.group_id, &pom.artifact_id)
                        );
                    }

                    if let Err(error) = crate::repository::utils::update_project_in_repositories(
                        &storage,
                        &repository.init_values,
                        format!("{}:{}", &pom.group_id, &pom.artifact_id),
                    )
                        .await
                    {
                        error!("Unable to update repository.json, {}", error);
                        trace!(
                            "Versvalueion {} Name: {}",
                            &pom.version,
                            format!("{}:{}", &pom.group_id, &pom.artifact_id)
                        );
                    }
                    let string = format!("{}/{}", project_folder, &pom.version);
                    let info = DeployInfo {
                        user: caller.clone(),
                        version: pom.version,
                        name: format!("{}:{}", &pom.group_id, &pom.artifact_id),
                        version_folder: string,
                    };

                    debug!("Starting Post Deploy Tasks");
                    if log_enabled!(Trace) {
                        trace!("Data {}", &info);
                    }
                    let deploy = handle_post_deploy(&storage, &repository.init_values, &info).await;
                    if let Err(error) = deploy {
                        error!("Error Handling Post Deploy Tasks {}", error);
                    } else {
                        debug!("All Post Deploy Tasks Completed and Happy :)");
                    }
                });
            }
        }
        Ok(RepoResponse::Ok)
    }
}

impl NitroRepository<MavenSettings> for MavenHandler {
    fn parse_project_to_directory<S: Into<String>>(value: S) -> String {
        value.into().replace('.', "/").replace(':', "/")
    }
}

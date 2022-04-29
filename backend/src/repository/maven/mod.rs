use crate::constants::PROJECT_FILE;
use actix_web::web::Bytes;
use actix_web::HttpRequest;
use log::Level::Trace;
use log::{debug, error, log_enabled, trace};

use crate::error::internal_error::InternalError;
use crate::repository::deploy::{handle_post_deploy, DeployInfo};
use crate::repository::maven::models::Pom;
use crate::repository::maven::utils::parse_project_to_directory;
use crate::repository::models::RepositorySummary;
use crate::repository::settings::security::Visibility;
use crate::repository::settings::Policy;

use crate::authentication::Authentication;
use crate::repository::types::RepoResponse::{BadRequest, NotFound, ProjectResponse};
use crate::repository::types::{Project, RepoResponse, RepoResult};
use crate::repository::types::{RDatabaseConnection, RepositoryRequest};
use crate::repository::utils::{
    get_project_data, get_version_data, get_versions, process_storage_files,
};
use crate::system::permissions::options::CanIDo;
use crate::system::user::UserModel;

pub mod models;
mod utils;

pub struct MavenHandler;

impl MavenHandler {
    pub async fn handle_get(
        request: &RepositoryRequest,
        http: &HttpRequest,
        conn: &RDatabaseConnection,
        auth: Authentication,
    ) -> RepoResult {
        if request.repository.security.visibility == Visibility::Private {
            let caller: UserModel = auth.get_user(conn).await??;
            caller.can_read_from(&request.repository)?;
        }

        let result = request
            .storage
            .get_file_as_response(&request.repository, &request.value, http)
            .await?;
        if let Some(result) = result {
            if result.is_left() {
                Ok(RepoResponse::FileResponse(result.left().unwrap()))
            } else {
                let vec = result.right().unwrap();
                let file_response = process_storage_files(
                    &request.storage,
                    &request.repository,
                    vec,
                    &request.value,
                )
                .await?;
                Ok(RepoResponse::NitroFileList(file_response))
            }
        } else {
            Ok(NotFound)
        }
    }

    pub async fn handle_put(
        request: &RepositoryRequest,
        _http: &HttpRequest,
        conn: &RDatabaseConnection,
        bytes: Bytes,
        auth: Authentication,
    ) -> RepoResult {
        let caller: UserModel = auth.get_user(conn).await??;
        caller.can_deploy_to(&request.repository)?;
        //TODO find a better way to do this
        match request.repository.settings.policy {
            Policy::Release => {
                if request.value.contains("-SNAPSHOT") {
                    return Ok(BadRequest("SNAPSHOT in release only".to_string()));
                }
            }
            Policy::Snapshot => {
                if !request.value.contains("-SNAPSHOT") {
                    return Ok(BadRequest("Release in a snapshot only".to_string()));
                }
            }
            Policy::Mixed => {}
        }
        request
            .storage
            .save_file(&request.repository, bytes.as_ref(), &request.value)
            .await?;
        if request.value.ends_with(".pom") {
            let vec = bytes.as_ref().to_vec();
            let result = String::from_utf8(vec)?;
            let pom: Result<Pom, serde_xml_rs::Error> = serde_xml_rs::from_str(&result);
            if let Err(error) = &pom {
                error!(
                    "Unable to Parse Pom File {} Error {}",
                    &request.value, error
                );
            }
            if let Ok(pom) = pom {
                let project_folder =
                    format!("{}/{}", pom.group_id.replace('.', "/"), pom.artifact_id);
                trace!("Project Folder Location {}", project_folder);
                let repository = request.repository.clone();
                let storage = request.storage.clone();
                actix_web::rt::spawn(async move {
                    if let Err(error) = utils::update_project(
                        &storage,
                        &repository,
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
                        &repository,
                        format!("{}:{}", &pom.group_id, &pom.artifact_id),
                    )
                    .await
                    {
                        error!("Unable to update repository.json, {}", error);
                        trace!(
                            "Version {} Name: {}",
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
                    let deploy = handle_post_deploy(&storage, &repository, &info).await;
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

    pub async fn handle_versions(
        request: &RepositoryRequest,
        _http: &HttpRequest,
        _conn: &RDatabaseConnection,
    ) -> RepoResult {
        let project_dir = parse_project_to_directory(&request.value);

        let vec = get_versions(&request.storage, &request.repository, project_dir).await?;
        Ok(RepoResponse::NitroVersionListingResponse(vec))
    }

    pub async fn handle_version(
        request: &RepositoryRequest,
        version: String,
        _http: &HttpRequest,
        _conn: &RDatabaseConnection,
    ) -> RepoResult {
        let project_dir = parse_project_to_directory(&request.value);

        let project_data =
            get_project_data(&request.storage, &request.repository, project_dir.clone()).await?;
        if let Some(project_data) = project_data {
            let version_data = crate::repository::utils::get_version_data(
                &request.storage,
                &request.repository,
                format!("{}/{}", project_dir, &version),
            )
            .await?;

            let project = Project {
                repo_summary: RepositorySummary::new(&request.repository),
                project: project_data,
                version: version_data,
                frontend_response: None,
            };
            return Ok(ProjectResponse(project));
        }
        RepoResult::Ok(NotFound)
    }

    pub async fn handle_project(
        request: &RepositoryRequest,
        _http: &HttpRequest,
        _conn: &RDatabaseConnection,
    ) -> RepoResult {
        let string = parse_project_to_directory(&request.value);

        let project_data =
            get_project_data(&request.storage, &request.repository, string.clone()).await?;
        if let Some(project_data) = project_data {
            let version_data = get_version_data(
                &request.storage,
                &request.repository,
                format!("{}/{}", string, &project_data.versions.latest_version),
            )
            .await?;

            let project = Project {
                repo_summary: RepositorySummary::new(&request.repository),
                project: project_data,
                version: version_data,
                frontend_response: None,
            };
            return Ok(ProjectResponse(project));
        }
        RepoResult::Ok(NotFound)
    }

    pub async fn latest_version(
        request: &RepositoryRequest,
        _http: &HttpRequest,
        _conn: &RDatabaseConnection,
    ) -> Result<Option<String>, InternalError> {
        let string = parse_project_to_directory(&request.value);
        let project_data = get_project_data(&request.storage, &request.repository, string).await?;
        Ok(if let Some(project_data) = project_data {
            let latest_release = project_data.versions.latest_release;
            if latest_release.is_empty() {
                Some(project_data.versions.latest_version)
            } else {
                Some(latest_release)
            }
        } else {
            None
        })
    }
}

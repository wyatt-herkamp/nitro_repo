use crate::constants::PROJECT_FILE;
use actix_web::web::Bytes;
use actix_web::HttpRequest;
use diesel::MysqlConnection;
use log::Level::Trace;
use log::{debug, error, log_enabled, trace};

use crate::error::internal_error::InternalError;
use crate::repository::deploy::{handle_post_deploy, DeployInfo};
use crate::repository::maven::models::Pom;
use crate::repository::maven::utils::parse_project_to_directory;
use crate::repository::models::{Policy, RepositorySummary};

use crate::repository::types::RepoResponse::{
    BadRequest, IAmATeapot, NotAuthorized, NotFound, ProjectResponse,
};
use crate::repository::types::RepositoryRequest;
use crate::repository::types::{Project, RepoResponse, RepoResult, RepositoryType};
use crate::repository::utils::{
    get_project_data, get_version_data, get_versions, process_storage_files,
};
use crate::system::utils::{can_deploy_basic_auth, can_read_basic_auth};

mod models;
mod utils;

pub struct MavenHandler;

impl RepositoryType for MavenHandler {
    fn handle_get(
        request: &RepositoryRequest,
        http: &HttpRequest,
        conn: &MysqlConnection,
    ) -> RepoResult {
        if !can_read_basic_auth(http.headers(), &request.repository, conn)? {
            return RepoResult::Ok(NotAuthorized);
        }
        let result =
            request
                .storage
                .get_file_as_response(&request.repository, &request.value, http)?;
        if result.is_left() {
            Ok(RepoResponse::FileResponse(result.left().unwrap()))
        } else {
            let vec = result.right().unwrap();
            if vec.is_empty() {
                return Ok(RepoResponse::NotFound);
            }
            let file_response =
                process_storage_files(&request.storage, &request.repository, vec, &request.value)?;
            Ok(RepoResponse::NitroFileList(file_response))
        }
    }

    fn handle_post(
        _request: &RepositoryRequest,
        _http: &HttpRequest,
        _conn: &MysqlConnection,
        _bytes: Bytes,
    ) -> RepoResult {
        Ok(IAmATeapot("Post is not handled in Maven".to_string()))
    }

    fn handle_put(
        request: &RepositoryRequest,
        http: &HttpRequest,
        conn: &MysqlConnection,
        bytes: Bytes,
    ) -> RepoResult {
        let x = can_deploy_basic_auth(http.headers(), &request.repository, conn)?;
        if !x.0 {
            return RepoResult::Ok(NotAuthorized);
        }

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
            .save_file(&request.repository, bytes.as_ref(), &request.value)?;
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
                    if let Err(error) = crate::repository::maven::utils::update_project(
                        &storage,
                        &repository,
                        &project_folder,
                        pom.version.clone(),
                        pom.clone(),
                    ) {
                        error!("Unable to update {}, {}", PROJECT_FILE, error);
                        if log_enabled!(Trace) {
                            trace!(
                                "Version {} Name: {}",
                                &pom.version,
                                format!("{}:{}", &pom.group_id, &pom.artifact_id)
                            );
                        }
                    }

                    if let Err(error) = crate::repository::utils::update_project_in_repositories(
                        &storage,
                        &repository,
                        format!("{}:{}", &pom.group_id, &pom.artifact_id),
                    ) {
                        error!("Unable to update repository.json, {}", error);
                        if log_enabled!(Trace) {
                            trace!(
                                "Version {} Name: {}",
                                &pom.version,
                                format!("{}:{}", &pom.group_id, &pom.artifact_id)
                            );
                        }
                    }
                    let string = format!("{}/{}", project_folder, &pom.version);
                    let info = DeployInfo {
                        user: x.1.unwrap(),
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

    fn handle_patch(
        _request: &RepositoryRequest,
        _http: &HttpRequest,
        _conn: &MysqlConnection,
        _bytes: Bytes,
    ) -> RepoResult {
        Ok(IAmATeapot("Patch is not handled in Maven".to_string()))
    }

    fn handle_head(
        request: &RepositoryRequest,
        http: &HttpRequest,
        _conn: &MysqlConnection,
    ) -> RepoResult {
        let result =
            request
                .storage
                .get_file_as_response(&request.repository, &request.value, http)?;
        if result.is_left() {
            Ok(RepoResponse::FileResponse(result.left().unwrap()))
        } else {
            Ok(RepoResponse::FileList(result.right().unwrap()))
        }
    }

    fn handle_versions(
        request: &RepositoryRequest,
        http: &HttpRequest,
        conn: &MysqlConnection,
    ) -> RepoResult {
        if !can_read_basic_auth(http.headers(), &request.repository, conn)? {
            return RepoResult::Ok(NotAuthorized);
        }
        let project_dir = parse_project_to_directory(&request.value);

        let vec = get_versions(&request.storage, &request.repository, project_dir)?;
        Ok(RepoResponse::NitroVersionListingResponse(vec))
    }

    fn handle_version(
        request: &RepositoryRequest,
        version: String,
        http: &HttpRequest,
        conn: &MysqlConnection,
    ) -> RepoResult {
        if !can_read_basic_auth(http.headers(), &request.repository, conn)? {
            return RepoResult::Ok(NotAuthorized);
        }
        let project_dir = parse_project_to_directory(&request.value);

        let project_data =
            get_project_data(&request.storage, &request.repository, project_dir.clone())?;
        if let Some(project_data) = project_data {
            let version_data = crate::repository::utils::get_version_data(
                &request.storage,
                &request.repository,
                format!("{}/{}", project_dir, &version),
            )?;

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

    fn handle_project(
        request: &RepositoryRequest,
        http: &HttpRequest,
        conn: &MysqlConnection,
    ) -> RepoResult {
        if !can_read_basic_auth(http.headers(), &request.repository, conn)? {
            return RepoResult::Ok(NotAuthorized);
        }
        let string = parse_project_to_directory(&request.value);

        let project_data = get_project_data(&request.storage, &request.repository, string.clone())?;
        if let Some(project_data) = project_data {
            let version_data = get_version_data(
                &request.storage,
                &request.repository,
                format!("{}/{}", string, &project_data.versions.latest_version),
            )?;

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

    fn latest_version(
        request: &RepositoryRequest,
        http: &HttpRequest,
        conn: &MysqlConnection,
    ) -> Result<Option<String>, InternalError> {
        if !can_read_basic_auth(http.headers(), &request.repository, conn)? {
            return Ok(None);
        }
        let string = parse_project_to_directory(&request.value);
        let project_data = get_project_data(&request.storage, &request.repository, string)?;
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

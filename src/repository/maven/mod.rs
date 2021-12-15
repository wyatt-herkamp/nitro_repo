use std::collections::HashMap;
use std::fmt::format;
use std::fs::{create_dir_all, File, OpenOptions, read_dir, read_to_string, remove_file};
use std::io::Write;

use actix_web::HttpRequest;
use actix_web::web::Bytes;
use diesel::MysqlConnection;
use log::{debug, error, log_enabled, trace};
use log::Level::Trace;

use crate::error::internal_error::InternalError;
use crate::repository::deploy::{DeployInfo, handle_post_deploy};
use crate::repository::maven::models::Pom;
use crate::repository::maven::utils::{get_latest_version, get_version, get_versions, parse_project_to_directory, update_project, update_project_in_repositories, update_versions};
use crate::repository::models::{Policy, RepositorySummary};
use crate::repository::repository::{
    Project, RepoResponse, RepoResult, RepositoryFile, RepositoryRequest, RepositoryType,
};
use crate::repository::repository::RepoResponse::{
    BadRequest, IAmATeapot, NotAuthorized, NotFound, ProjectResponse,
};
use crate::system::utils::{can_deploy_basic_auth, can_read_basic_auth};
use crate::utils::get_storage_location;

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

        let buf = get_storage_location()
            .join("storages")
            .join(&request.storage.name)
            .join(&request.repository.name)
            .join(&request.value);
        let path = format!(
            "{}/{}/{}",
            &request.storage.name, &request.repository.name, &request.value
        );

        if buf.exists() {
            if buf.is_dir() {
                let dir = read_dir(buf)?;
                let mut files = Vec::new();
                for x in dir {
                    let entry = x?;
                    let string = entry.file_name().into_string().unwrap();
                    let full = format!("{}/{}", path, &string);
                    let file = RepositoryFile {
                        name: string,
                        full_path: full,
                        directory: entry.file_type()?.is_dir(),
                        data: HashMap::new(),
                    };
                    files.push(file);
                }
                return Ok(RepoResponse::FileList(files));
            } else {
                return Ok(RepoResponse::FileResponse(buf));
            }
        }

        Ok(NotFound)
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
        let repo_location = get_storage_location()
            .join("storages")
            .join(&request.storage.name)
            .join(&request.repository.name);
        let buf = repo_location
            .join(&request.value);
        let parent = buf.parent().unwrap().to_path_buf();
        create_dir_all(&parent)?;

        if buf.exists() {
            remove_file(&buf)?;
        }
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .create(true)
            .open(&buf)?;
        file.write_all(bytes.as_ref())?;
        drop(file);
        if buf.to_str().unwrap().to_string().ends_with(".pom") {
            let result = read_to_string(&buf)?;
            let pom: Result<Pom, serde_xml_rs::Error> = serde_xml_rs::from_str(&result);
            if let Err(error) = &pom {
                error!("Unable to Parse Pom File {} Error {}", &buf.to_str().unwrap(),error);
            }
            if let Ok(pom) = pom {
                let project_folder = parent.parent().unwrap().to_path_buf();
                let repository = request.repository.clone();
                actix_web::rt::spawn(async move {
                    if let Err(error) = update_project(&project_folder) {
                        error!("Unable to update .nitro.project.json, {}", error);
                        if log_enabled!(Trace) {
                            trace!("Version {} Name: {}", &pom.version,format!("{}:{}", &pom.group_id, &pom.artifact_id));
                        }
                    }
                    if let Err(error) = update_versions(&project_folder, pom.version.clone()) {
                        error!("Unable to update .nitro.versions.json, {}", error);
                        if log_enabled!(Trace) {
                            trace!("Version {} Name: {}", &pom.version,format!("{}:{}", &pom.group_id, &pom.artifact_id));
                        }
                    }
                    if let Err(error) = update_project_in_repositories(format!("{}:{}", &pom.group_id, &pom.artifact_id), repo_location) {
                        error!("Unable to update repository.json, {}", error);
                        if log_enabled!(Trace) {
                            trace!("Version {} Name: {}", &pom.version,format!("{}:{}", &pom.group_id, &pom.artifact_id));
                        }
                    }
                    let info = DeployInfo {
                        user: x.1.unwrap(),
                        version: pom.version,
                        name: format!("{}:{}", &pom.group_id, &pom.artifact_id),
                        report_location: parent.join("report.json"),
                    };

                    debug!("Starting Post Deploy Tasks");
                    if log_enabled!(Trace) {
                        trace!("Data {}", &info);
                    }
                    let deploy = handle_post_deploy(&repository, &info).await;
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
        _http: &HttpRequest,
        _conn: &MysqlConnection,
    ) -> RepoResult {
        let buf = get_storage_location()
            .join("storages")
            .join(&request.storage.name)
            .join(&request.repository.name)
            .join(&request.value);
        let path = format!(
            "{}/{}/{}",
            &request.storage.name, &request.repository.name, &request.value
        );

        //TODO do not return the body
        if buf.exists() {
            if buf.is_dir() {
                let dir = read_dir(buf)?;
                let mut files = Vec::new();
                for x in dir {
                    let entry = x?;
                    let string = entry.file_name().into_string().unwrap();
                    let full = format!("{}/{}", path, &string);
                    let file = RepositoryFile {
                        name: string,
                        full_path: full,
                        directory: entry.file_type()?.is_dir(),
                        data: HashMap::new(),
                    };
                    files.push(file);
                }
                return Ok(RepoResponse::FileList(files));
            } else {
                return Ok(RepoResponse::FileResponse(buf));
            }
        }

        Ok(NotFound)
    }

    fn handle_versions(
        request: &RepositoryRequest,
        http: &HttpRequest,
        conn: &MysqlConnection,
    ) -> RepoResult {
        if !can_read_basic_auth(http.headers(), &request.repository, conn)? {
            return RepoResult::Ok(NotAuthorized);
        }
        let string = parse_project_to_directory(&request.value);

        let buf = get_storage_location()
            .join("storages")
            .join(&request.storage.name)
            .join(&request.repository.name)
            .join(&string);
        if !buf.exists() {
            return RepoResult::Ok(NotFound);
        }
        let vec = get_versions(&buf);
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
        let string = parse_project_to_directory(&request.value);

        let buf = get_storage_location()
            .join("storages")
            .join(&request.storage.name)
            .join(&request.repository.name)
            .join(string);
        if !buf.exists() {
            return RepoResult::Ok(NotFound);
        }
        let option = get_version(&buf, version);
        if option.is_none() {
            return Ok(RepoResponse::NotFound);
        }
        Ok(RepoResponse::NitroVersionResponse(option.unwrap()))
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
        let buf = get_storage_location()
            .join("storages")
            .join(&request.storage.name)
            .join(&request.repository.name)
            .join(&string);
        if !buf.exists() {
            return RepoResult::Ok(NotFound);
        }
        let vec = get_versions(&buf);
        let project = Project {
            repo_summary: RepositorySummary::new(&request.repository, &conn)?,
            versions: vec,
            frontend_response: None,
        };
        return Ok(ProjectResponse(project));
    }

    fn latest_version(
        request: &RepositoryRequest,
        http: &HttpRequest,
        conn: &MysqlConnection,
    ) -> Result<String, InternalError> {
        if !can_read_basic_auth(http.headers(), &request.repository, conn)? {
            return Ok("".to_string());
        }
        let string = parse_project_to_directory(&request.value);

        let buf = get_storage_location()
            .join("storages")
            .join(&request.storage.name)
            .join(&request.repository.name)
            .join(string);
        if !buf.exists() {
            return Ok("".to_string());
        }
        let vec = get_latest_version(&buf, false);
        Ok(vec.unwrap_or("".to_string()))
    }
}

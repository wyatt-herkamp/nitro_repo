use std::collections::HashMap;
use std::fs::{create_dir_all, OpenOptions, read_dir, remove_file};
use std::io::Write;

use actix_web::HttpRequest;
use actix_web::web::{Bytes};
use diesel::MysqlConnection;
use log::{debug, error};

use crate::error::internal_error::InternalError;
use crate::repository::deploy::{DeployInfo, handle_post_deploy};
use crate::repository::maven::utils::{get_latest_version, get_version, get_versions};
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
        let buf = get_storage_location()
            .join("storages")
            .join(&request.storage.name)
            .join(&request.repository.name)
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
        if buf.to_str().unwrap().to_string().ends_with(".pom") {
            let info = DeployInfo {
                user: x.1.unwrap(),
                version: "".to_string(),
                name: "SimpleAnnotation".to_string(),
                report_location: parent.join("report.json"),
            };
            let repository = request.repository.clone();
            debug!("Starting Post Deploy Tasks");
            actix_web::rt::spawn(async move {
                let deploy = handle_post_deploy(&repository, info).await;
                if let Err(error) = deploy {
                    error!("Error Handling Post Deploy Tasks {}", error);
                } else {
                    debug!("All Post Deploy Tasks Completed and Happy :)");
                }
            });
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
        let buf = get_storage_location()
            .join("storages")
            .join(&request.storage.name)
            .join(&request.repository.name)
            .join(&request.value);
        if !buf.exists() {
            return RepoResult::Ok(NotFound);
        }
        let vec = get_versions(&buf);
        Ok(RepoResponse::VersionListingResponse(vec))
    }

    fn handle_version(
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
        if !buf.exists() {
            return RepoResult::Ok(NotFound);
        }
        Ok(RepoResponse::VersionResponse(get_version(&buf)))
    }

    fn handle_project(
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
        let buf = get_storage_location()
            .join("storages")
            .join(&request.storage.name)
            .join(&request.repository.name)
            .join(&request.value);
        if !buf.exists() {
            return Ok("".to_string());
        }
        let vec = get_latest_version(&buf, false);
        Ok(vec)
    }
}

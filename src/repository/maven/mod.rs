mod models;
mod utils;

use crate::repository::repository::RepoResponse::{
    BadRequest, IAmATeapot, NotAuthorized, NotFound,
};
use crate::repository::repository::{
    RepoResponse, RepoResult, RepositoryFile, RepositoryRequest, RepositoryType,
};
use std::collections::HashMap;

use crate::system::utils::{can_deploy_basic_auth, can_read_basic_auth};

use actix_web::web::{Buf, Bytes};

use crate::error::internal_error::InternalError;
use actix_web::HttpRequest;
use diesel::MysqlConnection;
use std::fs::{create_dir_all, read_dir, remove_file, OpenOptions};
use std::io::Write;

use crate::repository::maven::utils::{get_latest_version, get_versions};
use crate::repository::models::Policy;
use crate::utils::get_storage_location;

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
        if !can_deploy_basic_auth(http.headers(), &request.repository, conn)? {
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
        create_dir_all(parent)?;

        if buf.exists() {
            remove_file(&buf)?;
        }
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .create(true)
            .open(buf)?;
        file.write_all(bytes.bytes())?;
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
        Ok(RepoResponse::VersionResponse(vec))
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

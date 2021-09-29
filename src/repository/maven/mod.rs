mod models;
mod utils;

use crate::repository::repository::RepoResponse::{BadRequest, NotAuthorized, NotFound};
use crate::repository::repository::{RepoResponse, RepoResult, RepositoryRequest, RepositoryType};

use crate::system::utils::{can_deploy_basic_auth, can_read_basic_auth};

use actix_web::web::{Buf, Bytes};

use diesel::MysqlConnection;
use std::fs::{create_dir_all, read_dir, remove_file, OpenOptions};
use std::io::Write;

use crate::error::request_error::RequestError;
use crate::repository::maven::utils::{get_latest_version, get_versions};
use crate::repository::models::Policy;
use crate::utils::get_storage_location;

pub struct MavenHandler;

impl RepositoryType for MavenHandler {
    fn handle_get(request: RepositoryRequest, conn: &MysqlConnection) -> RepoResult {
        if !can_read_basic_auth(request.request.headers(), &request.repository, conn)? {
            return RepoResult::Ok(NotAuthorized);
        }

        let buf = get_storage_location()
            .join("storages")
            .join(request.storage.name.clone())
            .join(request.repository.name.clone())
            .join(request.value);
        if buf.exists() {
            if buf.is_dir() {
                let dir = read_dir(buf)?;
                let mut files = Vec::new();
                for x in dir {
                    let entry = x?;
                    files.push(entry.file_name().into_string().unwrap());
                }
                return Ok(RepoResponse::FileList(files));
            } else {
                return Ok(RepoResponse::FileResponse(buf));
            }
        }

        return Ok(NotFound);
    }

    fn handle_post(
        _request: RepositoryRequest,
        _conn: &MysqlConnection,
        _bytes: Bytes,
    ) -> RepoResult {
        return Ok(BadRequest("Post is not handled in Maven".to_string()));
    }

    fn handle_put(request: RepositoryRequest, conn: &MysqlConnection, bytes: Bytes) -> RepoResult {
        if !can_deploy_basic_auth(request.request.headers(), &request.repository, conn)? {
            return RepoResult::Ok(NotAuthorized);
        }
        let path = request.value;
        //TODO find a better way to do this
        match request.repository.settings.policy {
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
        let buf = get_storage_location()
            .join("storages")
            .join(request.storage.name.clone())
            .join(request.repository.name.clone())
            .join(path.clone());
        let dir = buf.clone();
        let parent = dir.parent().unwrap().to_path_buf();
        create_dir_all(parent)?;

        if buf.exists() {
            remove_file(&buf)?;
        }
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .create(true)
            .open(buf)?;
        file.write_all(bytes.bytes());
        return Ok(RepoResponse::Ok);
    }

    fn handle_patch(
        _request: RepositoryRequest,
        _conn: &MysqlConnection,
        _bytes: Bytes,
    ) -> RepoResult {
        return Ok(BadRequest("Patch is not handled in Maven".to_string()));
    }

    fn handle_head(request: RepositoryRequest, _conn: &MysqlConnection) -> RepoResult {
        let buf = get_storage_location()
            .join("storages")
            .join(request.storage.name.clone())
            .join(request.repository.name.clone())
            .join(request.value);
        //TODO do not return the body
        if buf.exists() {
            if buf.is_dir() {
                let dir = read_dir(buf)?;
                let mut files = Vec::new();
                for x in dir {
                    let entry = x?;
                    files.push(entry.file_name().into_string().unwrap());
                }
                return Ok(RepoResponse::FileList(files));
            } else {
                return Ok(RepoResponse::FileResponse(buf));
            }
        }

        return Ok(NotFound);
    }

    fn handle_versions(request: RepositoryRequest, conn: &MysqlConnection) -> RepoResult {
        if !can_read_basic_auth(request.request.headers(), &request.repository, conn)? {
            return RepoResult::Ok(NotAuthorized);
        }
        let buf = get_storage_location()
            .join("storages")
            .join(request.storage.name.clone())
            .join(request.repository.name.clone())
            .join(request.value);
        if !buf.exists() {
            return RepoResult::Ok(NotFound);
        }
        let vec = get_versions(&buf);
        return Ok(RepoResponse::VersionResponse(vec));
    }

    fn latest_version(
        request: RepositoryRequest,
        conn: &MysqlConnection,
    ) -> Result<String, RequestError> {
        if !can_read_basic_auth(request.request.headers(), &request.repository, conn)? {
            return Ok("".to_string());
        }
        let buf = get_storage_location()
            .join("storages")
            .join(request.storage.name.clone())
            .join(request.repository.name.clone())
            .join(request.value);
        if !buf.exists() {
            return Ok("".to_string());
        }
        let vec = get_latest_version(&buf, false);
        return Ok(vec);
    }
}

use crate::repository::models::Repository;
use crate::repository::repository::RepoResponse::{NotAuthorized, NotFound};
use crate::repository::repository::{RepoResponse, RepoResult, RepositoryRequest, RepositoryType};
use crate::storage::models::Storage;
use crate::system::utils::can_deploy_basic_auth;
use actix_files::NamedFile;
use actix_web::web::{Buf, Bytes};
use actix_web::HttpRequest;
use diesel::MysqlConnection;
use std::fs::{create_dir_all, read_dir, remove_file, OpenOptions};
use std::io::Write;
use std::path::PathBuf;

pub struct MavenHandler;

impl RepositoryType for MavenHandler {
    fn handle_get(request: RepositoryRequest, conn: &MysqlConnection) -> RepoResult {
        let buf = PathBuf::new()
            .join("storages")
            .join(request.storage.name.clone())
            .join(request.repository.name.clone())
            .join(request.value);
        println!("{}-{}", buf.clone().to_str().unwrap(), buf.exists().clone());
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

    fn handle_post(request: RepositoryRequest, conn: &MysqlConnection, bytes: Bytes) -> RepoResult {
        return Ok(RepoResponse::Ok);
    }

    fn handle_put(request: RepositoryRequest, conn: &MysqlConnection, bytes: Bytes) -> RepoResult {
        if !can_deploy_basic_auth(request.request.headers(), &request.repository, conn)? {
            return RepoResult::Ok(NotAuthorized);
        }
        let buf = PathBuf::new()
            .join("storages")
            .join(request.storage.name.clone())
            .join(request.repository.name.clone())
            .join(request.value);
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
        request: RepositoryRequest,
        conn: &MysqlConnection,
        bytes: Bytes,
    ) -> RepoResult {
        return Ok(RepoResponse::Ok);
    }

    fn handle_head(request: RepositoryRequest, conn: &MysqlConnection) -> RepoResult {
        return Ok(RepoResponse::Ok);
    }
}

use crate::repository::repository::{RepositoryType, RepoResult, RepositoryRequest, RepoResponse};
use actix_web::HttpRequest;
use crate::storage::models::Storage;
use crate::repository::models::Repository;
use actix_web::web::Bytes;
use std::path::PathBuf;
use actix_files::NamedFile;
use crate::repository::repository::RepoResponse::NotFound;
use std::fs::read_dir;

pub struct MavenHandler;

impl RepositoryType for MavenHandler {
    fn handle_get(request: RepositoryRequest) -> RepoResult {
        let buf = PathBuf::new().join("storages").join(request.storage.name.clone()).join(request.repository.name.clone()).join(request.value);
        println!("{}-{}", buf.clone().to_str().unwrap(), buf.exists().clone());
        if buf.exists(){
            if buf.is_dir(){
                let dir = read_dir(buf)?;
                let mut files = Vec::new();
                for x in dir {
                    let entry = x?;
                    files.push(entry.file_name().into_string().unwrap());
                }
                return Ok(RepoResponse::FileList(files));

            }else{
              return  Ok(RepoResponse::FileResponse(buf));
            }
        }

        return Ok(NotFound)
    }

    fn handle_post(request: RepositoryRequest, bytes: Bytes) -> RepoResult {
        return Ok(RepoResponse::Ok);
    }

    fn handle_put(request: RepositoryRequest, bytes: Bytes) -> RepoResult {
        return Ok(RepoResponse::Ok);
    }

    fn handle_patch(request: RepositoryRequest, bytes: Bytes) -> RepoResult {
        return Ok(RepoResponse::Ok);
    }

    fn handle_head(request: RepositoryRequest) -> RepoResult {
        return Ok(RepoResponse::Ok);

    }
}
use crate::repository::repository::{RepositoryType, RepoResult, RepositoryRequest};
use actix_web::HttpRequest;
use crate::storage::models::Storage;
use crate::repository::models::Repository;
use actix_web::web::Bytes;

pub struct MavenHandler;

impl RepositoryType for MavenHandler {
    fn handle_get(request: RepositoryRequest) -> RepoResult {
        todo!()
    }

    fn handle_post(request: RepositoryRequest, bytes: Bytes) -> RepoResult {
        todo!()
    }

    fn handle_put(request: RepositoryRequest, bytes: Bytes) -> RepoResult {
        todo!()
    }

    fn handle_patch(request: RepositoryRequest, bytes: Bytes) -> RepoResult {
        todo!()
    }

    fn handle_head(request: RepositoryRequest) -> RepoResult {
        todo!()
    }
}
use crate::repository::repository::{RepositoryType, RepoResult};
use actix_web::HttpRequest;
use crate::storage::models::Storage;
use crate::repository::models::Repository;

pub struct MavenHandler;

impl RepositoryType for MavenHandler {
    fn handle_get(request: HttpRequest, storage: Storage, repository: Repository) -> RepoResult {
        todo!()
    }

    fn handle_post(request: HttpRequest, storage: Storage, repository: Repository) -> RepoResult {
        todo!()
    }

    fn handle_put(request: HttpRequest, storage: Storage, repository: Repository) -> RepoResult {
        todo!()
    }

    fn handle_patch(request: HttpRequest, storage: Storage, repository: Repository) -> RepoResult {
        todo!()
    }
}
use actix_web::{HttpRequest, HttpResponse};
use crate::storage::models::Storage;
use crate::repository::models::Repository;
use crate::repository::repo_error::RepositoryError;
pub type RepoResponse =HttpResponse;
pub type RepoResult = Result<RepoResponse, RepositoryError>;
pub trait RepositoryType{
    fn handle_get(request: HttpRequest, storage: Storage, repository: Repository) -> RepoResult;
    fn handle_post(request: HttpRequest, storage: Storage, repository: Repository) -> RepoResult;
    fn handle_put(request: HttpRequest, storage: Storage, repository: Repository) -> RepoResult;
    fn handle_patch(request: HttpRequest, storage: Storage, repository: Repository) -> RepoResult;
}
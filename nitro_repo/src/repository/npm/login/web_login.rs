use crate::repository::{
    npm::utils::NpmRegistryExt, RepoResponse, RepositoryHandlerError, RepositoryRequest,
};

use super::LoginResponse;

pub async fn perform_login(
    repository: &impl NpmRegistryExt,
    request: RepositoryRequest,
) -> Result<RepoResponse, RepositoryHandlerError> {
    return Ok(LoginResponse::UnsupportedLogin.into());
}

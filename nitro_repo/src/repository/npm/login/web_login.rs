use serde::{Deserialize, Serialize};

use crate::repository::{
    npm::{utils::NpmRegistryExt, NPMRegistryError},
    RepoResponse, RepositoryHandlerError, RepositoryRequest,
};

use super::LoginResponse;
#[derive(Debug, Serialize, Deserialize)]
pub struct WebLoginResponse {
    pub done_url: String,
    pub login_url: String,
}
pub async fn perform_login(
    repository: &impl NpmRegistryExt,
    request: RepositoryRequest,
) -> Result<RepoResponse, NPMRegistryError> {
    // TODO: Implement Web Login
    return Ok(LoginResponse::UnsupportedLogin.into());
}

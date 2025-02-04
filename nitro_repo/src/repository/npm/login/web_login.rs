use serde::{Deserialize, Serialize};

use crate::repository::{
    RepoResponse, RepositoryRequest,
    npm::{NPMRegistryError, utils::NpmRegistryExt},
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
    Ok(LoginResponse::UnsupportedLogin.into())
}

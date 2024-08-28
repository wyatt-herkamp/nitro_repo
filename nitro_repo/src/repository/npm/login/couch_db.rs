use std::fmt::Debug;

use chrono::{DateTime, FixedOffset};
use derive_more::derive::From;
use nr_core::{
    database::user::auth_token::NewRepositoryToken, user::permissions::RepositoryActions,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::{debug, instrument};

use crate::{
    app::authentication::verify_login,
    repository::{
        npm::{login::LoginResponse, utils::NpmRegistryExt},
        RepoResponse, RepositoryHandlerError, RepositoryRequest,
    },
};

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct CouchDBLoginRequest {
    pub name: String,
    pub password: String,
    pub email: Option<String>,
    #[serde(rename = "type")]
    pub login_type: String,
    #[serde(default)]
    pub roles: Vec<Value>,
    //#[serde(with = "nr_core::utils::time::iso_8601")]
    //pub date: DateTime<FixedOffset>,
}
impl Debug for CouchDBLoginRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CouchDBLogin")
            .field("name", &self.name)
            .field("password", &"********")
            .field("email", &self.email)
            .field("login_type", &self.login_type)
            .field("roles", &self.roles)
            //            .field("date", &self.date)
            .finish()
    }
}
#[derive(Debug, Serialize, Deserialize, From)]
pub struct CouchDBLoginResponse {
    pub token: String,
}
/// Handles the login request for CouchDB
/// Required route is `/-/user/org.couchdb.user:<username>`
#[instrument(name = "npm_couch_db_login")]
pub async fn perform_login(
    repository: &impl NpmRegistryExt,
    request: RepositoryRequest,
) -> Result<RepoResponse, RepositoryHandlerError> {
    let path_as_string = request.path.to_string();
    let Some(source) = request
        .user_agent_as_string()?
        .map(|header| format!("NPM CLI ({})", header))
    else {
        return Ok(RepoResponse::forbidden());
    };
    let user_name = path_as_string.replace("-/user/org.couchdb.user:", "");
    let body = request.body.body_as_string().await?;
    debug!(?user_name, ?body, "Handling PUT request");
    let login: CouchDBLoginRequest = serde_json::from_str(&body)?;
    debug!(?login, "Handling PUT request");
    let user = match verify_login(login.name, login.password, repository.site().as_ref()).await {
        Ok(ok) => ok,
        Err(err) => {
            return Ok(RepoResponse::forbidden());
        }
    };

    let (_, token) =
        NewRepositoryToken::new(user.id, source, repository.id(), RepositoryActions::all())
            .insert(repository.site().as_ref())
            .await?;
    return Ok(LoginResponse::ValidCouchDBLogin(CouchDBLoginResponse::from(token)).into());
}

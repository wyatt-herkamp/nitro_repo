//! NPM Registry Implementation
//!
//! Documentation for NPM: https://github.com/npm/registry/blob/main/docs/REGISTRY-API.md

use std::sync::Arc;

use crate::{
    app::{authentication::verify_login, responses::no_content_response, NitroRepo},
    repository::RepoResponse,
};
use axum::response::{IntoResponse, Response};
use derive_more::derive::Deref;
use http::StatusCode;
use nr_core::{
    database::{repository::DBRepository, user::auth_token::NewRepositoryToken},
    user::permissions::RepositoryActions,
};
use nr_storage::DynStorage;
use tracing::{debug, instrument};
use types::login::{CouchDBLoginRequest, CouchDBLoginResponse, LoginResponse};

use super::{Repository, RepositoryFactoryError};
pub mod registry_type;
pub mod types;
#[derive(derive_more::Debug)]
pub struct NpmRegistryInner {
    #[debug(skip)]
    pub site: NitroRepo,
    pub storage: DynStorage,
    pub id: uuid::Uuid,
    pub repository: DBRepository,
}
#[derive(Debug, Clone, Deref)]
pub struct NpmRegistry(Arc<NpmRegistryInner>);
impl NpmRegistry {
    pub async fn load(
        site: NitroRepo,
        storage: DynStorage,
        repository: DBRepository,
    ) -> Result<Self, RepositoryFactoryError> {
        Ok(Self(Arc::new(NpmRegistryInner {
            site,
            storage,
            id: repository.id,
            repository,
        })))
    }
}
impl Repository for NpmRegistry {
    fn get_storage(&self) -> DynStorage {
        self.0.storage.clone()
    }

    fn get_type(&self) -> &'static str {
        "npm"
    }

    fn config_types(&self) -> Vec<&str> {
        vec![]
    }

    fn name(&self) -> String {
        self.0.repository.name.to_string()
    }

    fn id(&self) -> uuid::Uuid {
        self.id
    }

    fn visibility(&self) -> nr_core::repository::Visibility {
        nr_core::repository::Visibility::Public
    }

    fn is_active(&self) -> bool {
        true
    }

    #[instrument(name = "NpmRegistry::handle_post")]
    async fn handle_post(
        &self,
        request: super::RepositoryRequest,
    ) -> Result<super::RepoResponse, super::RepositoryHandlerError> {
        let headers = request.headers();
        debug!(?headers, "Handling POST request");
        let path_as_string = request.path.to_string();
        if path_as_string.starts_with(r#"-/user/org\.couchdb\.user:"#) {
            let user_name = path_as_string.replace("-/user/org.couchdb.user:", "");
        }
        Ok(no_content_response().into())
    }
    #[instrument(name = "NpmRegistry::handle_put")]
    async fn handle_put(
        &self,
        request: super::RepositoryRequest,
    ) -> Result<super::RepoResponse, super::RepositoryHandlerError> {
        let headers = request.headers();
        debug!(?headers, "Handling PUT request");

        let path_as_string = request.path.to_string();
        debug!(?path_as_string, "Handling PUT request");
        if path_as_string.starts_with(r#"-/user/org.couchdb.user:"#) {
            let user_name = path_as_string.replace("-/user/org.couchdb.user:", "");
            let body = request.body.body_as_string().await?;
            debug!(?user_name, ?body, "Handling PUT request");
            let login: CouchDBLoginRequest = serde_json::from_str(&body)?;
            debug!(?login, "Handling PUT request");
            let message = format!("user '{}' created", login.name);
            let user = match verify_login(login.name, login.password, self.site.as_ref()).await {
                Ok(ok) => ok,
                Err(err) => {
                    return Ok(RepoResponse::forbidden());
                }
            };
            let (_, token) = NewRepositoryToken::new(
                user.id,
                "NPM CLI".to_owned(),
                self.id,
                RepositoryActions::all(),
            )
            .insert(self.site.as_ref())
            .await?;
            return Ok(LoginResponse::ValidCouchDBLogin(CouchDBLoginResponse::from(token)).into());
        } else if path_as_string.eq("/-/v1/login") {
            // TODO: Implement the new login system
            return Ok(LoginResponse::UnsupportedLogin.into());
        }
        Ok(no_content_response().into())
    }
}

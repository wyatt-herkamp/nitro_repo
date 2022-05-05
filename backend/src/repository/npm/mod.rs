use std::collections::HashMap;

use actix_web::web::Bytes;

use actix_web::http::header::HeaderMap;
use actix_web::http::StatusCode;

use log::{debug, error, trace};
use regex::Regex;
use sea_orm::DatabaseConnection;
use std::string::String;

use crate::repository::npm::models::{Attachment, LoginRequest, LoginResponse, PublishRequest};
use crate::repository::npm::utils::generate_get_response;

use crate::authentication::{verify_login, Authentication};

use crate::repository::response::RepoResponse;
use crate::storage::models::Storage;
use crate::system::permissions::options::CanIDo;
use crate::system::user::UserModel;

pub mod error;
pub mod models;
mod utils;

use crate::repository::data::RepositoryConfig;
use crate::repository::handler::RepositoryHandler;
use crate::repository::nitro::nitro_repository::NitroRepositoryHandler;

use crate::api_response::{APIError, APIResponse};
use crate::error::internal_error::InternalError;
use async_trait::async_trait;
use tokio::sync::RwLockReadGuard;

pub struct NPMHandler<'a> {
    config: RepositoryConfig,
    storage: RwLockReadGuard<'a, Box<dyn Storage>>,
}

impl<'a> NPMHandler<'a> {
    pub fn create(
        repository: RepositoryConfig,
        storage: RwLockReadGuard<'a, Box<dyn Storage>>,
    ) -> NPMHandler<'a> {
        NPMHandler {
            config: repository,
            storage,
        }
    }
    fn bad_npm_command() -> APIError {
        APIResponse::from(("Bad NPM Command", StatusCode::BAD_REQUEST)).into()
    }
}
// name/version
#[async_trait]
impl<'a> RepositoryHandler<'a> for NPMHandler<'a> {
    async fn handle_get(
        &self,
        path: &str,
        headers: &HeaderMap,
        conn: &DatabaseConnection,
        authentication: Authentication,
    ) -> Result<RepoResponse, APIError> {
        let caller: UserModel = authentication.get_user(conn).await??;
        if let Some(value) = caller.can_read_from(&self.config)? {
            return Err(value.into());
        }
        if headers.get("npm-command").is_some() {
            if path.contains(".tgz") {
                let split: Vec<&str> = path.split("/-/").collect();
                let package = split.get(0).unwrap().to_string();
                let file = split.get(1).unwrap().to_string();
                let version = file
                    .replace(format!("{}-", &package).as_str(), "")
                    .replace(".tgz", "");
                let nitro_file_location = format!("{package}/{version}/{file}");
                debug!(
                    "Trying to Retrieve Package: {} Version {}. Location {}",
                    &package, &version, &nitro_file_location
                );

                let result = self
                    .storage
                    .get_file_as_response(&self.config, &nitro_file_location)
                    .await?;
                if let Some(_result) = result {
                    //TODO Handle StorageFileResponse
                } else {
                    return Ok(RepoResponse::Response(APIResponse::ok()));
                }
            }
            let get_response = generate_get_response(&self.storage, &self.config, path)
                .await
                .unwrap();
            if get_response.is_none() {
                return Ok(RepoResponse::Response(APIResponse::ok()));
            }
            let string = serde_json::to_value(&get_response.unwrap())?;
            Ok(RepoResponse::Json(string, StatusCode::OK))
        } else {
            let result = self
                .storage
                .get_file_as_response(&self.config, path)
                .await
                .map_err(InternalError::from)?;
            if let Some(_result) = result {
                //TODO Handle StorageFileResponse
                todo!("UN Handled Result Type")
            } else {
                return Ok(RepoResponse::Response(APIResponse::ok()));
            }
        }
    }

    async fn handle_put(
        &self,

        path: &str,
        headers: &HeaderMap,
        conn: &DatabaseConnection,
        authentication: Authentication,
        bytes: Bytes,
    ) -> Result<RepoResponse, APIError> {
        let user = Regex::new(r"-/user/org\.couchdb\.user:[a-zA-Z]+").unwrap();
        // Check if its a user verification request
        if user.is_match(path) {
            let content = String::from_utf8(bytes.as_ref().to_vec()).unwrap();
            let json: LoginRequest = serde_json::from_str(content.as_str()).unwrap();
            let username = path.replace("-/user/org.couchdb.user:", "");
            let user = verify_login(username, json.password, conn).await??;
            trace!("User Request for {} was authorized", &user.username);
            let message = format!("user '{}' created", user.username);
            let created_response = serde_json::to_value(&LoginResponse { ok: message })
                .map_err(InternalError::from)?;
            Ok(RepoResponse::Json(created_response, StatusCode::CREATED));
        }
        //Handle Normal Request
        let caller: UserModel = authentication.get_user(conn).await??;
        if let Some(value) = caller.can_deploy_to(&self.config)? {
            return Err(value.into());
        }
        if let Some(npm_command) = headers.get("npm-command") {
            let npm_command = npm_command.to_str().unwrap();
            trace!("NPM {} Command {}", &path, &npm_command);
            if npm_command.eq("publish") {
                let publish_request: PublishRequest =
                    serde_json::from_slice(bytes.as_ref()).map_err(APIResponse::bad_request)?;

                let attachments: HashMap<String, serde_json::Result<Attachment>> = publish_request
                    ._attachments
                    .iter()
                    .map(|(key, path)| {
                        let attachment: serde_json::Result<Attachment> =
                            serde_json::from_value(path.clone()).map_err(APIResponse::bad_request);
                        (key.clone(), attachment)
                    })
                    .collect();
                let mut exists = false;
                for (attachment_key, attachment) in attachments {
                    let attachment = attachment?;
                    let attachment_data =
                        base64::decode(attachment.data).map_err(APIResponse::bad_request)?;
                    for (version, version_data) in publish_request.versions.iter() {
                        let version_data_string = serde_json::to_string(version_data)
                            .map_err(APIResponse::bad_request)?;
                        trace!(
                            "Publishing {} Version: {} File:{} Data {}",
                            &publish_request.name,
                            version,
                            &attachment_key,
                            &version_data_string
                        );
                        let attachment_file_loc =
                            format!("{}/{}/{}", &publish_request.name, version, &attachment_key);
                        let npm_version_data =
                            format!("{}/{}/package.json", &publish_request.name, version);

                        if self
                            .storage
                            .save_file(&self.config, attachment_data.as_ref(), &attachment_file_loc)
                            .await
                            .map_err(InternalError::from)?
                        {
                            exists = true
                        };

                        if self
                            .storage
                            .save_file(
                                &self.config,
                                version_data_string.as_bytes(),
                                &npm_version_data,
                            )
                            .await
                            .map_err(InternalError::from)?
                        {
                            exists = true;
                        };

                        let project_folder = publish_request.name.clone();
                        let version_folder =
                            format!("{}/{}", &project_folder, &version_data.version);

                        trace!("Project Folder Location {}", project_folder);
                        let version_for_saving = version_data.clone();
                        let user = caller.clone();
                        if let Err(error) = self
                            .post_deploy(
                                project_folder,
                                version_folder,
                                user,
                                version_for_saving.into(),
                            )
                            .await
                        {
                            error!("Unable to complete post processing Tasks {}", error);
                        }
                    }
                }

                return Ok(RepoResponse::PUTResponse(exists));
            }
            Err(NPMHandler::bad_npm_command())
        } else {
            Err(NPMHandler::bad_npm_command())
        }
    }
}

impl NitroRepositoryHandler for NPMHandler<'_> {
    fn parse_project_to_directory<S: Into<String>>(path: S) -> String {
        path.into().replace('.', "/").replace(':', "/")
    }

    fn storage(&self) -> &Box<dyn Storage> {
        &self.storage
    }

    fn repository(&self) -> &RepositoryConfig {
        &self.config
    }
}

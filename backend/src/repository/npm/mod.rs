use std::collections::HashMap;

use crate::constants::PROJECT_FILE;
use actix_web::web::Bytes;

use actix_web::http::header::HeaderMap;
use actix_web::http::StatusCode;
use log::Level::Trace;
use log::{debug, error, log_enabled, trace};
use regex::Regex;
use sea_orm::DatabaseConnection;
use std::string::String;

use crate::repository::deploy::{handle_post_deploy, DeployInfo};

use crate::repository::npm::models::{Attachment, LoginRequest, LoginResponse, NPMSettings, PublishRequest, Version};
use crate::repository::npm::utils::{generate_get_response, is_valid};

use crate::authentication::Authentication;

use crate::repository::response::RepoResponse;
use crate::repository::response::RepoResponse::{
    BadRequest, CreatedWithJSON, NotAuthorized, NotFound,
};
use crate::storage::models::Storage;
use crate::system::permissions::options::CanIDo;
use crate::system::user::UserModel;

pub mod error;
pub mod models;
mod utils;

use crate::repository::data::RepositoryConfig;
use crate::repository::error::RepositoryError;
use crate::repository::handler::RepositoryHandler;
use crate::repository::nitro::nitro_repository::NitroRepository;
use crate::repository::nitro::utils::update_project_in_repositories;
use async_trait::async_trait;

pub struct NPMHandler;

// name/version
#[async_trait]
impl RepositoryHandler<NPMSettings> for NPMHandler {
    async fn handle_get(
        repository: &RepositoryConfig<NPMSettings>,
        storage: &Storage,
        path: &str,
        headers: &HeaderMap,
        conn: &DatabaseConnection,
        authentication: Authentication,
    ) -> Result<RepoResponse, RepositoryError> {
        let caller: UserModel = authentication.get_user(conn).await??;
        if let Some(value) = caller.can_read_from(repository)? {
            return Err(RepositoryError::RequestError(
                value.to_string(),
                StatusCode::UNAUTHORIZED,
            ));
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

                let result = storage
                    .get_file_as_response(&repository, &nitro_file_location)
                    .await?;
                if let Some(_result) = result {
                    //TODO Handle StorageFileResponse
                } else {
                    return Ok(NotFound);
                }
            }
            let get_response = generate_get_response(storage, &repository, path)
                .await
                .unwrap();
            if get_response.is_none() {
                return Ok(NotFound);
            }
            let string = serde_json::to_string_pretty(&get_response.unwrap())?;
            Ok(RepoResponse::OkWithJSON(string))
        } else {
            let result = storage.get_file_as_response(&repository, path).await?;
            if let Some(_result) = result {
                //TODO Handle StorageFileResponse
                todo!("UN Handled Result Type")
            } else {
                Ok(NotFound)
            }
        }
    }

    async fn handle_put(
        repository: &RepositoryConfig<NPMSettings>,
        storage: &Storage,
        path: &str,
        headers: &HeaderMap,
        conn: &DatabaseConnection,
        authentication: Authentication,
        bytes: Bytes,
    ) -> Result<RepoResponse, RepositoryError> {
        let user = Regex::new(r"-/user/org\.couchdb\.user:[a-zA-Z]+").unwrap();
        // Check if its a user verification request
        if user.is_match(path) {
            let content = String::from_utf8(bytes.as_ref().to_vec()).unwrap();
            let json: LoginRequest = serde_json::from_str(content.as_str()).unwrap();
            let username = path.replace("-/user/org.couchdb.user:", "");
            return if is_valid(&username, &json, conn).await? {
                trace!("User Request for {} was authorized", &username);
                let message = format!("user '{}' created", username);
                Ok(CreatedWithJSON(serde_json::to_string(&LoginResponse {
                    ok: message,
                })?))
            } else {
                trace!("User Request for {} was not authorized", &username);
                Ok(NotAuthorized)
            };
        }
        //Handle Normal Request
        let caller: UserModel = authentication.get_user(conn).await??;
        if let Some(value) = caller.can_deploy_to(repository)? {
            return Err(RepositoryError::RequestError(
                value.to_string(),
                StatusCode::UNAUTHORIZED,
            ));
        }
        if let Some(npm_command) = headers.get("npm-command") {
            let npm_command = npm_command.to_str().unwrap();
            trace!("NPM {} Command {}", &path, &npm_command);
            if npm_command.eq("publish") {
                let publish_request: PublishRequest = serde_json::from_slice(bytes.as_ref())?;

                let attachments: HashMap<String, serde_json::Result<Attachment>> = publish_request
                    ._attachments
                    .iter()
                    .map(|(key, path)| {
                        let attachment: serde_json::Result<Attachment> =
                            serde_json::from_value(path.clone());
                        (key.clone(), attachment)
                    })
                    .collect();
                for (attachment_key, attachment) in attachments {
                    let attachment = attachment?;
                    let attachment_data = base64::decode(attachment.data).map_err(|error| {
                        RepositoryError::RequestError(error.to_string(), StatusCode::BAD_REQUEST)
                    })?;
                    for (version, version_data) in publish_request.versions.iter() {
                        let version_data_string = serde_json::to_string(version_data)?;
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

                        storage
                            .save_file(&repository, attachment_data.as_ref(), &attachment_file_loc)
                            .await?;

                        storage
                            .save_file(
                                &repository,
                                version_data_string.as_bytes(),
                                &npm_version_data,
                            )
                            .await?;

                        let project_folder = publish_request.name.clone();
                        trace!("Project Folder Location {}", project_folder);
                        let repository = repository.clone();
                        let storage = storage.clone();
                        let version_for_saving = version_data.clone();
                        let user = caller.clone();
                        actix_web::rt::spawn(NPMHandler::post_deploy(storage, repository, user, project_folder, version_for_saving));
                    }
                }

                return Ok(RepoResponse::Ok);
            }
            Ok(BadRequest(format!("Bad Request {}", npm_command)))
        } else {
            Ok(BadRequest("Missing NPM-Command".to_string()))
        }
    }
}

impl NitroRepository<NPMSettings> for NPMHandler {
    fn parse_project_to_directory<S: Into<String>>(path: S) -> String {
        path.into().replace('.', "/").replace(':', "/")
    }
}

impl NPMHandler {
    async fn post_deploy(storage: Storage, repository: RepositoryConfig<NPMSettings>, user: UserModel, project_folder: String, version_for_saving: Version) {
        if let Err(error) = crate::repository::npm::utils::update_project(
            &storage,
            &repository,
            &project_folder,
            version_for_saving.clone(),
        )
            .await
        {
            error!("Unable to update {}, {}", PROJECT_FILE, error);
            if log_enabled!(Trace) {
                trace!(
                                        "Version {} Name: {}",
                                        &version_for_saving.version,
                                        &version_for_saving.name
                                    );
            }
        }

        if let Err(error) = update_project_in_repositories(
            &storage,
            &repository,
            version_for_saving.name.clone(),
        )
            .await
        {
            error!("Unable to update repository.json, {}", error);
            if log_enabled!(Trace) {
                trace!(
                                        "Version {} Name: {}",
                                        &version_for_saving.version,
                                        &version_for_saving.name
                                    );
            }
        }
        let info = DeployInfo {
            user: user.clone(),
            version: version_for_saving.version.clone(),
            name: version_for_saving.name.clone(),
            version_folder: format!(
                "{}/{}",
                &project_folder, &version_for_saving.version
            ),
        };

        debug!("Starting Post Deploy Tasks");
        if log_enabled!(Trace) {
            trace!("Data {}", &info);
        }
        let deploy = handle_post_deploy(&storage, &repository, &info).await;
        if let Err(error) = deploy {
            error!("Error Handling Post Deploy Tasks {}", error);
        } else {
            debug!("All Post Deploy Tasks Completed and Happy :)");
        }
    }
}
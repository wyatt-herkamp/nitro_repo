use std::collections::HashMap;


use std::string::String;
use crate::constants::PROJECT_FILE;
use actix_web::web::Bytes;
use actix_web::HttpRequest;
use diesel::MysqlConnection;
use log::Level::Trace;
use log::{debug, error, log_enabled, trace};
use regex::Regex;

use crate::error::internal_error::InternalError;
use crate::repository::deploy::{handle_post_deploy, DeployInfo};
use crate::repository::models::RepositorySummary;

use crate::repository::npm::models::{
    Attachment, LoginRequest, LoginResponse, PublishRequest,
};
use crate::repository::npm::utils::{generate_get_response, get_version_data, is_valid, parse_project_to_directory};

use crate::repository::types::RepoResponse::{BadRequest, CreatedWithJSON, IAmATeapot, NotAuthorized, NotFound, ProjectResponse};
use crate::repository::types::{
    Project, RepoResponse, RepoResult, RepositoryRequest, RepositoryType,
};
use crate::repository::utils::{get_project_data, get_version, get_versions, process_storage_files};
use crate::system::utils::{can_deploy_basic_auth, can_read_basic_auth};

mod models;
mod utils;

pub struct NPMHandler;

// name/version
impl RepositoryType for NPMHandler {
    fn handle_get(
        request: &RepositoryRequest,
        http: &HttpRequest,
        conn: &MysqlConnection,
    ) -> RepoResult {
        if !can_read_basic_auth(http.headers(), &request.repository, conn)? {
            return RepoResult::Ok(NotAuthorized);
        }
        if http.headers().get("npm-command").is_some() {
            if request.value.contains(".tgz") {
                let split: Vec<&str> = request.value.split("/-/").collect();
                let package = split.get(0).unwrap().to_string();
                let file = split.get(1).unwrap().to_string();
                let version = file.replace(format!("{}-", &package).as_str(), "").replace(".tgz", "");
                let nitro_file_location = format!("{package}/{version}/{file}");
                debug!("Trying to Retrieve Package: {} Version {}. Location {}", &package, &version,&nitro_file_location);
                let result =
                    request
                        .storage
                        .get_file_as_response(&request.repository, &nitro_file_location, http)?;
                return if result.is_left() {
                    Ok(RepoResponse::FileResponse(result.left().unwrap()))
                } else {
                    Ok(BadRequest("Expected File got Folder".to_string()))
                };
            }
            let get_response = generate_get_response(&request.storage, &request.repository, &request.value).unwrap();
            if get_response.is_none() {
                return Ok(NotFound);
            }
            let string = serde_json::to_string_pretty(&get_response.unwrap())?;
            Ok(RepoResponse::OkWithJSON(string))
        } else {
            let result =
                request
                    .storage
                    .get_file_as_response(&request.repository, &request.value, http)?;
            if result.is_left() {
                Ok(RepoResponse::FileResponse(result.left().unwrap()))
            } else {
                let vec = result.right().unwrap();
                if vec.is_empty() {
                    return Ok(RepoResponse::NotFound);
                }
                let file_response = process_storage_files(&request.storage, &request.repository, vec, &request.value)?;
                Ok(RepoResponse::NitroFileList(file_response))
            }
        }
    }

    fn handle_post(
        _request: &RepositoryRequest,
        _http: &HttpRequest,
        _conn: &MysqlConnection,
        _bytes: Bytes,
    ) -> RepoResult {
        Ok(IAmATeapot("POST is not handled in NPM".to_string()))
    }

    fn handle_put(
        request: &RepositoryRequest,
        http: &HttpRequest,
        conn: &MysqlConnection,
        bytes: Bytes,
    ) -> RepoResult {
        for x in http.headers() {
            log::trace!("Header {}: {}", x.0, x.1.to_str().unwrap());
        }
        log::trace!("URL: {}", request.value);
        let user = Regex::new(r"-/user/org\.couchdb\.user:[a-zA-Z]+").unwrap();
        // Check if its a user verification request
        if user.is_match(request.value.as_str()) {
            let content = String::from_utf8(bytes.as_ref().to_vec()).unwrap();
            let json: LoginRequest = serde_json::from_str(content.as_str()).unwrap();
            let username = request.value.replace("-/user/org.couchdb.user:", "");
            return if is_valid(&username, &json, conn)? {
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
        let (allowed, user) = can_deploy_basic_auth(http.headers(), &request.repository, conn)?;
        if !allowed {
            return RepoResult::Ok(NotAuthorized);
        }
        let user = user.unwrap();
        if let Some(npm_command) = http.headers().get("npm-command") {
            let npm_command = npm_command.to_str().unwrap();
            trace!("NPM {} Command {}",&request.value, &npm_command);
            if npm_command.eq("publish") {
                let publish_request: PublishRequest = serde_json::from_slice(bytes.as_ref())?;
                let attachments: HashMap<String, serde_json::Result<Attachment>> = publish_request._attachments.iter().map(|(key, value)| {
                    let attachment: serde_json::Result<Attachment> = serde_json::from_value(value.clone());
                    (key.clone(), attachment)
                }).collect();
                for (attachment_key, attachment) in attachments {
                    let attachment = attachment?;
                    let attachment_data = base64::decode(attachment.data)?;
                    for (version, version_data) in publish_request.versions.iter() {
                        let version_data_string = serde_json::to_string(version_data)?;
                        trace!("Publishing {} Version: {} File:{} Data {}",&publish_request.name, version, &attachment_key, &version_data_string);
                        let attachment_file_loc = format!("{}/{}/{}", &publish_request.name, version, &attachment_key);
                        let npm_version_data = format!("{}/{}/package.json", &publish_request.name, version);
                        request.storage.save_file(&request.repository, attachment_data.as_ref(), &attachment_file_loc)?;
                        request.storage.save_file(&request.repository, version_data_string.as_bytes(), &npm_version_data)?;

                        let project_folder = publish_request.name.clone();
                        trace!("Project Folder Location {}", project_folder);
                        let repository = request.repository.clone();
                        let storage = request.storage.clone();
                        let version_for_saving = version_data.clone();
                        let user = user.clone();
                        actix_web::rt::spawn(async move {
                            if let Err(error) = crate::repository::npm::utils::update_project(
                                &storage,
                                &repository,
                                &project_folder,
                                version_for_saving.clone(),
                            ) {
                                error!("Unable to update {}, {}", PROJECT_FILE, error);
                                if log_enabled!(Trace) {
                                    trace!(
                                "Version {} Name: {}",
                                &version_for_saving.version,
                                &version_for_saving.name
                            );
                                }
                            }

                            if let Err(error) = crate::repository::utils::update_project_in_repositories(
                                &storage,
                                &repository,
                                version_for_saving.name.clone(),
                            ) {
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
                                version_folder: format!("{}/{}", &project_folder, &version_for_saving.version),
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
                        });
                    }
                }


                return Ok(RepoResponse::Ok);
            }
            Ok(BadRequest(format!("Bad Request {}", npm_command)))
        } else {
            Ok(BadRequest("Missing NPM-Command".to_string()))
        }
    }

    fn handle_patch(
        _request: &RepositoryRequest,
        _http: &HttpRequest,
        _conn: &MysqlConnection,
        _bytes: Bytes,
    ) -> RepoResult {
        Ok(IAmATeapot("Patch is not handled in NPM".to_string()))
    }

    fn handle_head(
        _request: &RepositoryRequest,
        _http: &HttpRequest,
        _conn: &MysqlConnection,
    ) -> RepoResult {
        Ok(IAmATeapot("HEAD is not handled in NPM".to_string()))
    }

    fn handle_versions(
        request: &RepositoryRequest,
        http: &HttpRequest,
        conn: &MysqlConnection,
    ) -> RepoResult {
        if !can_read_basic_auth(http.headers(), &request.repository, conn)? {
            return RepoResult::Ok(NotAuthorized);
        }

        let vec = get_versions(&request.storage, &request.repository, request.value.clone())?;
        Ok(RepoResponse::NitroVersionListingResponse(vec))
    }

    fn handle_version(
        request: &RepositoryRequest,
        version: String,
        http: &HttpRequest,
        conn: &MysqlConnection,
    ) -> RepoResult {
        if !can_read_basic_auth(http.headers(), &request.repository, conn)? {
            return RepoResult::Ok(NotAuthorized);
        }
        let project_dir = parse_project_to_directory(&request.value);

        let project_data = get_project_data(&request.storage, &request.repository, project_dir.clone())?;
        if let Some(project_data) = project_data {
            let version_data = crate::repository::utils::get_version_data(&request.storage, &request.repository, format!("{}/{}", project_dir, &version))?;

            let project = Project {
                repo_summary: RepositorySummary::new(&request.repository),
                project: project_data,
                version: version_data,
                frontend_response: None,
            };
            return Ok(ProjectResponse(project));
        }
        RepoResult::Ok(NotFound)
    }

    fn handle_project(
        request: &RepositoryRequest,
        http: &HttpRequest,
        conn: &MysqlConnection,
    ) -> RepoResult {
        for x in http.headers() {
            log::trace!("Header {}: {}", x.0, x.1.to_str().unwrap());
        }
        log::trace!("URL: {}", request.value);
        if !can_read_basic_auth(http.headers(), &request.repository, conn)? {
            return RepoResult::Ok(NotAuthorized);
        }
        let project_dir = parse_project_to_directory(&request.value);

        let project_data = get_project_data(&request.storage, &request.repository, project_dir.clone())?;
        if let Some(project_data) = project_data {
            let version_data = crate::repository::utils::get_version_data(&request.storage, &request.repository, format!("{}/{}", project_dir, &project_data.versions.latest_version))?;

            let project = Project {
                repo_summary: RepositorySummary::new(&request.repository),
                project: project_data,
                version: version_data,
                frontend_response: None,
            };
            return Ok(ProjectResponse(project));
        }
        RepoResult::Ok(NotFound)
    }

    fn latest_version(
        request: &RepositoryRequest,
        http: &HttpRequest,
        conn: &MysqlConnection,
    ) -> Result<Option<String>, InternalError> {
        if !can_read_basic_auth(http.headers(), &request.repository, conn)? {
            return Ok(None);
        }
        let project_dir = parse_project_to_directory(&request.value);

        let project_data = get_project_data(&request.storage, &request.repository, project_dir)?;
        Ok(if let Some(project_data) = project_data {
            let latest_release = project_data.versions.latest_release;
            if latest_release.is_empty() {
                Some(project_data.versions.latest_version)
            } else {
                Some(latest_release)
            }
        } else {
            None
        })
    }
}
use std::collections::HashMap;
use std::fs::{create_dir_all, read_dir, remove_file, File};
use std::io::{BufReader, Write};
use std::string::String;

use actix_web::web::Bytes;
use actix_web::HttpRequest;
use diesel::MysqlConnection;
use log::Level::Trace;
use log::{debug, error, log_enabled, trace};
use regex::Regex;
use serde_json::Error;

use crate::error::internal_error::InternalError;
use crate::repository::deploy::{handle_post_deploy, DeployInfo};
use crate::repository::models::RepositorySummary;

use crate::repository::npm::models::{
    Attachment, GetResponse, LoginRequest, LoginResponse, PublishRequest, Version,
};
use crate::repository::npm::utils::{is_valid};

use crate::repository::types::RepoResponse::{BadRequest, CreatedWithJSON, FileResponse, IAmATeapot, NotAuthorized, NotFound, ProjectResponse};
use crate::repository::types::{
    Project, RepoResponse, RepoResult, RepositoryFile, RepositoryRequest, RepositoryType,
};
use crate::repository::utils::{get_versions,
};
use crate::system::utils::{can_deploy_basic_auth, can_read_basic_auth};
use crate::utils::get_storage_location;

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
        for x in http.headers() {
            log::trace!("Header {}: {}", x.0, x.1.to_str().unwrap());
        }
        log::trace!("URL: {}", request.value);
        if let Some(npm_command) = http.headers().get("npm-command") {
            trace!("NPM {} Command {}",&request.value, http.headers().get("npm-command").unwrap().to_str().unwrap());
            return Ok(RepoResponse::Ok);
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
                Ok(RepoResponse::FileList(vec))
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
        let result1 = can_deploy_basic_auth(http.headers(), &request.repository, conn)?;
        if !result1.0 {
            return RepoResult::Ok(NotAuthorized);
        }

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
                    }
                }
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

        todo!()
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
        todo!()
    }

    fn handle_project(
        request: &RepositoryRequest,
        http: &HttpRequest,
        conn: &MysqlConnection,
    ) -> RepoResult {
        if !can_read_basic_auth(http.headers(), &request.repository, conn)? {
            return RepoResult::Ok(NotAuthorized);
        }
        todo!()
    }

    fn latest_version(
        request: &RepositoryRequest,
        http: &HttpRequest,
        conn: &MysqlConnection,
    ) -> Result<Option<String>, InternalError> {
        if !can_read_basic_auth(http.headers(), &request.repository, conn)? {
            return Ok(Some("401".to_string()));
        }

        todo!()
    }
}
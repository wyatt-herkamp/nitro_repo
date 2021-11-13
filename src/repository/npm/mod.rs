use std::fmt::format;
use std::fs::{create_dir_all, OpenOptions, remove_file};
use std::io::Write;
use std::string;
use std::string::String;

use actix_web::HttpRequest;
use actix_web::web::{Buf, Bytes};
use diesel::MysqlConnection;
use regex::Regex;
use serde_json::Value;

use crate::error::internal_error::InternalError;
use crate::repository::npm::auth::is_valid;
use crate::repository::npm::models::{Attachment, LoginRequest, LoginResponse, PublishRequest};
use crate::repository::repository::{RepoResponse, RepoResult, RepositoryRequest, RepositoryType};
use crate::repository::repository::RepoResponse::{Created_With_JSON, IAmATeapot, NotAuthorized};
use crate::system::utils::can_deploy_basic_auth;
use crate::utils::get_storage_location;

mod models;
mod auth;

pub struct NPMHandler;

impl RepositoryType for NPMHandler {
    fn handle_get(request: &RepositoryRequest, http: &HttpRequest, conn: &MysqlConnection) -> RepoResult {
        println!("GET {}", &request.value);
        for x in http.headers() {
            println!("Header {}: {}", &x.0, &x.1.to_str().unwrap())
        }
        return Ok(RepoResponse::Ok);
    }

    fn handle_post(request: &RepositoryRequest, http: &HttpRequest, conn: &MysqlConnection, bytes: Bytes) -> RepoResult {
        println!("POST {}", &request.value);
        for x in http.headers() {
            println!("Header {}: {}", &x.0, &x.1.to_str().unwrap())
        }
        println!("{}", String::from_utf8(bytes.bytes().to_vec()).unwrap());
        return Ok(NotAuthorized);
    }

    fn handle_put(request: &RepositoryRequest, http: &HttpRequest, conn: &MysqlConnection, bytes: Bytes) -> RepoResult {
        let user = Regex::new(r"-/user/org\.couchdb\.user:[a-zA-Z]+").unwrap();
        if user.is_match(request.value.as_str()) {
            let content = String::from_utf8(bytes.bytes().to_vec()).unwrap();
            let json: LoginRequest = serde_json::from_str(content.as_str()).unwrap();
            let username = request.value.replace("-/user/org.couchdb.user:", "");
            return if is_valid(&username.to_string(), &json, &conn)? {
                let message = format!("user '{}' created", username);
                Ok(Created_With_JSON(serde_json::to_string(&LoginResponse { ok: message })?))
            } else {
                println!("Five Unauthorized");
                Ok(NotAuthorized)
            };
        }
        if !can_deploy_basic_auth(http.headers(), &request.repository, conn)? {
            return RepoResult::Ok(NotAuthorized);
        }
        let value: PublishRequest = serde_json::from_slice(bytes.bytes()).unwrap();
        println!("{}", serde_json::to_string_pretty(&value)?);
        let buf = get_storage_location()
            .join("storages")
            .join(&request.storage.name)
            .join(&request.repository.name);
        for x in value._attachments {
            let file = buf.join(x.0);
            let parent = file.parent().unwrap().to_path_buf();
            create_dir_all(parent)?;

            if file.exists() {
                remove_file(&file).unwrap();
            }
            println!("{:?}", &file);
            let mut file = OpenOptions::new()
                .write(true)
                .create_new(true)
                .create(true)
                .open(file).unwrap();
            let result: Attachment = serde_json::from_value(x.1).unwrap();
            let attachment = base64::decode(result.data)?;

            file.write_all(&attachment).unwrap();
        }
        println!("PUT {}", &request.value);
        for x in http.headers() {
            println!("Header {}: {}", &x.0, &x.1.to_str().unwrap())
        }
        return Ok(RepoResponse::Ok);
    }

    fn handle_patch(request: &RepositoryRequest, http: &HttpRequest, conn: &MysqlConnection, bytes: Bytes) -> RepoResult {
        Ok(IAmATeapot("Patch is not handled in NPM".to_string()))
    }

    fn handle_head(request: &RepositoryRequest, http: &HttpRequest, conn: &MysqlConnection) -> RepoResult {
        Ok(IAmATeapot("HEAD is not handled in NPM".to_string()))
    }

    fn handle_versions(request: &RepositoryRequest, http: &HttpRequest, conn: &MysqlConnection) -> RepoResult {
        todo!()
    }

    fn handle_version(request: &RepositoryRequest, http: &HttpRequest, conn: &MysqlConnection) -> RepoResult {
        todo!()
    }

    fn handle_project(request: &RepositoryRequest, http: &HttpRequest, conn: &MysqlConnection) -> RepoResult {
        todo!()
    }

    fn latest_version(request: &RepositoryRequest, http: &HttpRequest, conn: &MysqlConnection) -> Result<String, InternalError> {
        todo!()
    }
}
use std::collections::HashMap;
use std::fmt::format;
use std::fs::{create_dir_all, File, OpenOptions, remove_file};
use std::io::{BufReader, Write};
use std::iter::Map;
use std::string;
use std::string::String;

use actix_web::HttpRequest;
use actix_web::web::{Buf, Bytes};
use diesel::MysqlConnection;
use futures_util::AsyncWriteExt;
use regex::Regex;
use serde_json::Value;

use crate::error::internal_error::InternalError;
use crate::repository::npm::auth::is_valid;
use crate::repository::npm::models::{Attachment, get_latest_version, GetResponse, LoginRequest, LoginResponse, PublishRequest, Version};
use crate::repository::repository::{RepoResponse, RepoResult, RepositoryRequest, RepositoryType};
use crate::repository::repository::RepoResponse::{Created_With_JSON, FileResponse, IAmATeapot, NotAuthorized};
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


        let buf = get_storage_location()
            .join("storages")
            .join(&request.storage.name)
            .join(&request.repository.name);
        if request.value.ends_with(".tgz") {
            let mut id = buf;
            for x in request.value.split("/-/").collect::<Vec<&str>>().first().unwrap().split("/") {
                id = id.join(x);
            };
            let vec = request.value.split("/").collect::<Vec<&str>>();
            let file_name = vec.last().unwrap().to_string();
            let file = id.join("attachments").join(file_name);
            println!("{:?}", &file);
            return Ok(FileResponse(file));
            //Get File
        } else {
            let buf = buf.join(&request.value.replace("%2f", "/"));
            // Return Info Response
            let files = buf.join("version-*");
            let pattern = format!("{}", files.to_str().unwrap());
            println!("{}", &pattern);
            let result = glob::glob(pattern.as_str()).unwrap();
            let mut versions = HashMap::new();
            for x in result {
                let buf1 = x.unwrap();
                let version: Version = serde_json::from_reader(BufReader::new(File::open(buf1)?))?;
                versions.insert(version.version.clone(), version);
            }
            let string = serde_json::to_string_pretty(&versions)?;
            let times_json = buf.join("times.json");
            let times: HashMap<String, String> = serde_json::from_reader(File::open(times_json)?)?;
            let latest_version = get_latest_version(&times);
            let version = versions.get(&latest_version).unwrap().clone();
            let response = GetResponse {
                id: version.name.clone(),
                name: version.name.clone(),
                other: version.other.clone(),
                versions,
                times,
            };
            println!("{}", &string);
            return Ok(RepoResponse::Ok_With_JSON(serde_json::to_string(&response)?));
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
        let buf = get_storage_location()
            .join("storages")
            .join(&request.storage.name)
            .join(&request.repository.name);
        let artifact = buf.join(&value.name);
        create_dir_all(&artifact)?;

        for x in value._attachments {
            let attachments = artifact.join("attachments");
            create_dir_all(&attachments);

            let name = x.0.split("/").collect::<Vec<&str>>().get(1).unwrap().to_string();
            let file = attachments.join(name);
            if file.exists() {
                remove_file(&file).unwrap();
            }
            let mut file = OpenOptions::new()
                .write(true)
                .create_new(true)
                .create(true)
                .open(file).unwrap();
            let result: Attachment = serde_json::from_value(x.1).unwrap();
            let attachment = base64::decode(result.data)?;

            file.write_all(&attachment)?;
        }

        let times_json = artifact.join("times.json");
        if !times_json.exists() {
            let time = chrono::Utc::now();
            let time = time.format("%Y-%m-%dT%H:%M:%S.%3fZ").to_string();
            let mut file = File::create(&times_json)?;
            let mut times = HashMap::new();
            times.insert("created", time.clone());
            file.write_all(serde_json::to_string(&times)?.as_bytes());
        }
        for (key, value) in value.versions {
            let version = artifact.join(format!("version-{}.json", &key));
            if version.exists() {
                remove_file(&version);
            }
            let result = serde_json::to_string_pretty(&value)?;
            let mut file = OpenOptions::new()
                .write(true)
                .create_new(true)
                .create(true)
                .open(version)?;
            file.write_all(result.as_bytes())?;

            //Append Time
            let time = chrono::Utc::now();
            let time = time.format("%Y-%m-%dT%H:%M:%S.%3fZ").to_string();
            let mut times_map: HashMap<String, String> = serde_json::from_reader(File::open(&times_json)?)?;
            remove_file(&times_json);
            times_map.insert(key.clone(), time);
            let mut times_json = File::create(&times_json)?;
            times_json.write_all(serde_json::to_string_pretty(&times_map)?.as_bytes());
        }
        // Everything is a ok.... OR is IT??????
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
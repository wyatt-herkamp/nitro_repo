use std::collections::HashMap;
use std::fs::{create_dir_all, read_dir, remove_file, File, OpenOptions};
use std::io::{BufReader, Write};
use std::string::String;

use actix_web::web::Bytes;
use actix_web::HttpRequest;
use diesel::MysqlConnection;
use regex::Regex;

use crate::error::internal_error::InternalError;
use crate::repository::models::RepositorySummary;
use crate::repository::npm::auth::is_valid;
use crate::repository::npm::models::{
    get_latest_version, Attachment, GetResponse, LoginRequest, LoginResponse, PublishRequest,
    Version,
};
use crate::repository::repository::RepoResponse::{
    CreatedWithJSON, FileResponse, IAmATeapot, NotAuthorized, VersionResponse,
};
use crate::repository::repository::Version as RepoVersion;
use crate::repository::repository::{
    Project, RepoResponse, RepoResult, RepositoryFile, RepositoryRequest, RepositoryType,
};
use crate::repository::utils::{build_artifact_directory, build_directory};
use crate::system::utils::{can_deploy_basic_auth, can_read_basic_auth};

mod auth;
mod models;
mod utils;

pub struct NPMHandler;

impl RepositoryType for NPMHandler {
    fn handle_get(
        request: &RepositoryRequest,
        http: &HttpRequest,
        conn: &MysqlConnection,
    ) -> RepoResult {
        if !can_read_basic_auth(http.headers(), &request.repository, conn)? {
            return RepoResult::Ok(NotAuthorized);
        }
        if http.headers().contains_key("npm-command") {
            let buf = build_directory(&request);
            if request.value.ends_with(".tgz") {
                let mut id = buf;
                for x in request
                    .value
                    .split("/-/")
                    .collect::<Vec<&str>>()
                    .first()
                    .unwrap()
                    .split("/")
                {
                    id = id.join(x);
                }
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
                    let version: Version =
                        serde_json::from_reader(BufReader::new(File::open(buf1)?))?;
                    versions.insert(version.version.clone(), version);
                }
                let string = serde_json::to_string_pretty(&versions)?;
                let times = crate::repository::npm::utils::read_time_file(
                    &request.storage,
                    &request.repository,
                    &request.value,
                )?;
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
                return Ok(RepoResponse::OkWithJSON(serde_json::to_string(&response)?));
            }
        } else {
            let buf = build_artifact_directory(&request);
            let path = format!(
                "{}/{}/{}",
                &request.storage.name, &request.repository.name, &request.value
            );

            if buf.exists() {
                if buf.is_dir() {
                    let dir = read_dir(buf)?;
                    let mut files = Vec::new();
                    for x in dir {
                        let entry = x?;
                        let string = entry.file_name().into_string().unwrap();
                        let full = format!("{}/{}", path, &string);
                        let file = RepositoryFile {
                            name: string,
                            full_path: full,
                            directory: entry.file_type()?.is_dir(),
                            data: HashMap::new(),
                        };
                        files.push(file);
                    }
                    return Ok(RepoResponse::FileList(files));
                } else {
                    return Ok(RepoResponse::FileResponse(buf));
                }
            }
        }
        return Ok(RepoResponse::Ok);
    }

    fn handle_post(
        _request: &RepositoryRequest,
        _http: &HttpRequest,
        _conn: &MysqlConnection,
        _bytes: Bytes,
    ) -> RepoResult {
        return Ok(NotAuthorized);
    }

    fn handle_put(
        request: &RepositoryRequest,
        http: &HttpRequest,
        conn: &MysqlConnection,
        bytes: Bytes,
    ) -> RepoResult {
        let user = Regex::new(r"-/user/org\.couchdb\.user:[a-zA-Z]+").unwrap();
        if user.is_match(request.value.as_str()) {
            let content = String::from_utf8(bytes.as_ref().to_vec()).unwrap();
            let json: LoginRequest = serde_json::from_str(content.as_str()).unwrap();
            let username = request.value.replace("-/user/org.couchdb.user:", "");
            return if is_valid(&username.to_string(), &json, &conn)? {
                let message = format!("user '{}' created", username);
                Ok(CreatedWithJSON(serde_json::to_string(&LoginResponse {
                    ok: message,
                })?))
            } else {
                println!("Five Unauthorized");
                Ok(NotAuthorized)
            };
        }
        let result1 = can_deploy_basic_auth(http.headers(), &request.repository, conn)?;
        if !result1.0 {
            return RepoResult::Ok(NotAuthorized);
        }
        let value: PublishRequest = serde_json::from_slice(bytes.as_ref()).unwrap();
        let buf = build_directory(&request);
        let artifact = buf.join(&value.name);
        create_dir_all(&artifact)?;

        for x in value._attachments {
            let attachments = artifact.join("attachments");
            create_dir_all(&attachments)?;

            let name =
                x.0.split("/")
                    .collect::<Vec<&str>>()
                    .get(1)
                    .unwrap()
                    .to_string();
            let file = attachments.join(name);
            if file.exists() {
                remove_file(&file).unwrap();
            }
            let mut file = OpenOptions::new()
                .write(true)
                .create_new(true)
                .create(true)
                .open(file)
                .unwrap();
            let result: Attachment = serde_json::from_value(x.1).unwrap();
            let attachment = base64::decode(result.data)?;

            file.write_all(&attachment)?;
        }

        let times_json = crate::repository::npm::utils::get_time_file(
            &request.storage,
            &request.repository,
            &request.value,
        );

        if !times_json.exists() {
            let time = chrono::Utc::now();
            let time = time.format("%Y-%m-%dT%H:%M:%S.%3fZ").to_string();
            let mut file = File::create(&times_json)?;
            let mut times = HashMap::new();
            times.insert("created", time.clone());
            file.write_all(serde_json::to_string(&times)?.as_bytes())?;
        }
        for (key, value) in value.versions {
            let version = artifact.join(format!("version-{}.json", &key));
            if version.exists() {
                remove_file(&version)?;
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
            let mut times_map: HashMap<String, String> =
                serde_json::from_reader(File::open(&times_json)?)?;
            remove_file(&times_json)?;
            times_map.insert(key.clone(), time);
            let mut times_json = File::create(&times_json)?;
            times_json.write_all(serde_json::to_string_pretty(&times_map)?.as_bytes())?;
        }
        // Everything is a ok.... OR is IT??????
        return Ok(RepoResponse::Ok);
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
        _http: &HttpRequest,
        _conn: &MysqlConnection,
    ) -> RepoResult {
        let times_map = crate::repository::npm::utils::read_time_file(
            &request.storage,
            &request.repository,
            &request.value,
        )?;

        let mut versions = Vec::new();
        for x in times_map {
            versions.push(RepoVersion {
                version: x.0,
                other: HashMap::new(),
            });
        }
        return Ok(RepoResponse::VersionListingResponse(versions));
    }

    fn handle_version(
        request: &RepositoryRequest,
        _http: &HttpRequest,
        _conn: &MysqlConnection,
    ) -> RepoResult {
        return Ok(VersionResponse(RepoVersion {
            version: request.value.split("/").last().unwrap().to_string(),
            other: HashMap::new(),
        }));
    }

    fn handle_project(
        request: &RepositoryRequest,
        _http: &HttpRequest,
        conn: &MysqlConnection,
    ) -> RepoResult {
        let times_map = crate::repository::npm::utils::read_time_file(
            &request.storage,
            &request.repository,
            &request.value,
        )?;

        let mut versions = Vec::new();
        for x in times_map {
            versions.push(RepoVersion {
                version: x.0,
                other: HashMap::new(),
            });
        }
        let project = Project {
            repo_summary: RepositorySummary::new(&request.repository, &conn)?,
            versions: versions,
            frontend_response: None,
        };
        return Ok(RepoResponse::ProjectResponse(project));
    }

    fn latest_version(
        request: &RepositoryRequest,
        _http: &HttpRequest,
        _conn: &MysqlConnection,
    ) -> Result<String, InternalError> {
        let times_map = crate::repository::npm::utils::read_time_file(
            &request.storage,
            &request.repository,
            &request.value,
        )?;
        Ok(get_latest_version(&times_map))
    }
}

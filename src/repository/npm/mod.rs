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

use crate::error::internal_error::InternalError;
use crate::repository::deploy::{handle_post_deploy, DeployInfo};
use crate::repository::models::RepositorySummary;

use crate::repository::npm::auth::is_valid;
use crate::repository::npm::models::{
    Attachment, GetResponse, LoginRequest, LoginResponse, PublishRequest, Version,
};
use crate::repository::npm::utils::get_version;

use crate::repository::types::RepoResponse::{
    CreatedWithJSON, FileResponse, IAmATeapot, NotAuthorized, NotFound, ProjectResponse,
};
use crate::repository::types::{
    Project, RepoResponse, RepoResult, RepositoryFile, RepositoryRequest, RepositoryType,
};
use crate::repository::utils::{build_artifact_directory, build_directory, get_latest_version, get_project_data, get_versions};
use crate::system::utils::{can_deploy_basic_auth, can_read_basic_auth};
use crate::utils::get_storage_location;

mod auth;
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
        if http.headers().contains_key("npm-command") {
            let buf = build_directory(request);
            if request.value.ends_with(".tgz") {
                let mut id = buf;
                for x in request
                    .value
                    .split("/-/")
                    .collect::<Vec<&str>>()
                    .first()
                    .unwrap()
                    .split('/')
                {
                    id = id.join(x);
                }
                let vec = request.value.split('/').collect::<Vec<&str>>();
                let file_name = vec.last().unwrap().to_string();
                let file = id.join("attachments").join(file_name);
                println!("{:?}", &file);
                return Ok(FileResponse(file));
                //Get File
            } else {
                let buf = buf.join(&request.value.replace("%2f", "/"));
                // Return Info Response
                let files = buf.join("version-*");
                let pattern = files.to_str().unwrap().to_string();
                println!("{}", &pattern);
                let result = glob::glob(pattern.as_str()).unwrap();
                let mut npm_versions = HashMap::new();
                for x in result {
                    let buf1 = x.unwrap();
                    let version: Version =
                        serde_json::from_reader(BufReader::new(File::open(buf1)?))?;
                    npm_versions.insert(version.version.clone(), version);
                }
                let nitro_versions = get_versions(&buf);
                let latest_version = get_latest_version(&buf, true).unwrap();
                let version = npm_versions.get(&latest_version).unwrap();
                let response = GetResponse {
                    id: version.name.clone(),
                    name: version.name.clone(),
                    other: version.other.clone(),
                    versions: npm_versions,
                    times: nitro_versions.into(),
                    dist_tags: latest_version.into(),
                };
                return Ok(RepoResponse::OkWithJSON(serde_json::to_string(&response)?));
            }
        } else {
            let buf = build_artifact_directory(request);
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
        Ok(RepoResponse::Ok)
    }

    fn handle_post(
        _request: &RepositoryRequest,
        _http: &HttpRequest,
        _conn: &MysqlConnection,
        _bytes: Bytes,
    ) -> RepoResult {
        Ok(NotAuthorized)
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
            return if is_valid(&username, &json, conn)? {
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
        let repo_location = build_directory(request);
        let artifact = repo_location.join(&value.name);
        create_dir_all(&artifact)?;

        for x in value._attachments {
            let attachments = artifact.join("attachments");
            create_dir_all(&attachments)?;

            let name = if x.0.contains('/') {
                x.0.split('/')
                    .collect::<Vec<&str>>()
                    .get(1)
                    .unwrap()
                    .to_string()
            } else {
                x.0.clone()
            };
            let file = attachments.join(name);
            if file.exists() {
                remove_file(&file).unwrap();
            }
            let mut file = File::create(&file)?;
            let result: Attachment = serde_json::from_value(x.1).unwrap();
            let attachment = base64::decode(result.data)?;

            file.write_all(&attachment)?;
        }
        let user1 = result1.1.unwrap();

        for (key, value) in value.versions {
            let version = artifact.join(format!("version-{}.json", &key));
            if version.exists() {
                remove_file(&version)?;
            }
            let result = serde_json::to_string_pretty(&value)?;
            let mut file = File::create(&version)?;
            file.write_all(result.as_bytes())?;

            let repo_location = build_directory(request);
            let project_folder = artifact.clone();
            let repository1 = request.repository.clone();
            let user2 = user1.clone();

            actix_web::rt::spawn(async move {
                if let Err(error) =
                    crate::repository::npm::utils::update_project(&project_folder, key.clone())
                {
                    error!("Unable to update .nitro.project.json, {}", error);
                    if log_enabled!(Trace) {
                        trace!("Version {} Name: {}", &key, &value.name);
                    }
                }
                if let Err(error) = crate::repository::utils::update_project_in_repositories(
                    value.name.clone(),
                    repo_location,
                ) {
                    error!("Unable to update repository.json, {}", error);
                    if log_enabled!(Trace) {
                        trace!("Version {} Name: {}", &key, &value.name);
                    }
                }
                let info = DeployInfo {
                    user: user2,
                    version: value.version.clone(),
                    name: value.name.clone(),
                    report_location: project_folder.join(format!("report-{}.json", value.version)),
                };

                debug!("Starting Post Deploy Tasks");
                if log_enabled!(Trace) {
                    trace!("Data {}", &info);
                }
                let deploy = handle_post_deploy(&repository1, &info).await;
                if let Err(error) = deploy {
                    error!("Error Handling Post Deploy Tasks {}", error);
                } else {
                    debug!("All Post Deploy Tasks Completed and Happy :)");
                }
            });
        }
        // Everything is a ok.... OR is IT??????
        Ok(RepoResponse::Ok)
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

        let buf = get_storage_location()
            .join("storages")
            .join(&request.storage.name)
            .join(&request.repository.name)
            .join(&request.value);
        if !buf.exists() {
            return RepoResult::Ok(NotFound);
        }
        let vec = get_versions(&buf);
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

        let buf = get_storage_location()
            .join("storages")
            .join(&request.storage.name)
            .join(&request.repository.name)
            .join(&request.value);
        if !buf.exists() {
            return RepoResult::Ok(NotFound);
        }
        let option = get_version(&buf, version);
        if option.is_none() {
            return Ok(RepoResponse::NotFound);
        }
        Ok(RepoResponse::NitroVersionResponse(option.unwrap()))
    }

    fn handle_project(
        request: &RepositoryRequest,
        http: &HttpRequest,
        conn: &MysqlConnection,
    ) -> RepoResult {
        if !can_read_basic_auth(http.headers(), &request.repository, conn)? {
            return RepoResult::Ok(NotAuthorized);
        }
        let buf = get_storage_location()
            .join("storages")
            .join(&request.storage.name)
            .join(&request.repository.name)
            .join(&request.value);
        if !buf.exists() {
            return RepoResult::Ok(NotFound);
        }
        let project_data = get_project_data(&buf)?;
        if project_data.is_none(){
            return RepoResult::Ok(NotFound);

        }
        let project = Project {
            repo_summary: RepositorySummary::new(&request.repository, conn)?,
            frontend_response: None,
            project: project_data.unwrap()
        };
        Ok(ProjectResponse(project))
    }

    fn latest_version(
        request: &RepositoryRequest,
        http: &HttpRequest,
        conn: &MysqlConnection,
    ) -> Result<String, InternalError> {
        if !can_read_basic_auth(http.headers(), &request.repository, conn)? {
            return Ok("".to_string());
        }

        let buf = get_storage_location()
            .join("storages")
            .join(&request.storage.name)
            .join(&request.repository.name)
            .join(&request.value);
        if !buf.exists() {
            return Ok("".to_string());
        }
        let vec = get_latest_version(&buf, false);
        Ok(vec.unwrap_or_else(||"".to_string()))
    }
}

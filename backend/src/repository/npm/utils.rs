use std::collections::HashMap;

use log::{trace, warn};

use chrono::{DateTime, NaiveDateTime, Utc};

use crate::error::internal_error::InternalError;
use crate::repository::nitro::{NitroRepoVersions, ProjectData, VersionData};

use crate::repository::utils::get_project_data;
use crate::utils::get_current_time;

use crate::constants::{PROJECT_FILE, VERSION_DATA};
use crate::repository::models::Repository;
use argon2::{Argon2, PasswordHash, PasswordVerifier};
use sea_orm::DatabaseConnection;

use crate::repository::npm::models::{
    DistTags, GetResponse, LoginRequest, NPMTimes, NPMVersions, Version,
};
use crate::storage::models::StringStorage;
use crate::system::auth_token::Relation::User;

pub fn is_valid(
    username: &str,
    request: &LoginRequest,
    conn: &DatabaseConnection,
) -> Result<bool, InternalError> {
    //TODO
    let result1 = None;
    if result1.is_none() {
        return Ok(false);
    }
    let argon2 = Argon2::default();
    let user = result1.unwrap();
    let parsed_hash = PasswordHash::new(user.password.as_str())?;
    if argon2
        .verify_password(request.password.as_bytes(), &parsed_hash)
        .is_err()
    {
        return Ok(false);
    }
    Ok(true)
}

static NPM_TIME_FORMAT: &str = "%Y-%m-%dT%H:%M:%S.%3fZ";

pub fn format_time(time: i64) -> String {
    let naive = NaiveDateTime::from_timestamp(time, 0);
    let date_time: DateTime<Utc> = DateTime::from_utc(naive, Utc);
    date_time.format(NPM_TIME_FORMAT).to_string()
}

impl From<NitroRepoVersions> for HashMap<String, String> {
    fn from(value: NitroRepoVersions) -> Self {
        let mut map = HashMap::new();
        for x in value.versions {
            let naive = NaiveDateTime::from_timestamp(x.time, 0);
            let date_time: DateTime<Utc> = DateTime::from_utc(naive, Utc);
            let format = date_time.format(NPM_TIME_FORMAT).to_string();
            map.insert(x.version, format);
        }
        map
    }
}

pub fn update_project(
    storage: &StringStorage,
    repository: &Repository,
    project_folder: &str,
    version: Version,
) -> Result<(), InternalError> {
    let project_file = format!("{}/{}", project_folder, PROJECT_FILE);
    let version_folder = format!("{}/{}/{}", &project_folder, &version.version, VERSION_DATA);

    trace!("Project File Location {}", project_file);
    let option = storage.get_file(repository, &project_file)?;
    let mut project_data: ProjectData = if let Some(data) = option {
        let string = String::from_utf8(data)?;
        let value = serde_json::from_str(&string)?;
        storage.delete_file(repository, &project_file)?;
        value
    } else {
        ProjectData {
            versions: Default::default(),
            created: get_current_time(),
            updated: get_current_time(),
        }
    };
    project_data.updated = get_current_time();
    let horrible_line_of_code = if let Some(desc) = version.other.get("description") {
        desc.as_str().unwrap().to_string()
    } else {
        "".to_string()
    };

    let version_data = VersionData {
        name: version.name,
        description: horrible_line_of_code,
        source: None,
        licence: None,
        version: version.version.clone(),
        created: get_current_time(),
    };
    project_data.versions.update_version(version.version);
    storage.save_file(
        repository,
        serde_json::to_string_pretty(&project_data)?.as_bytes(),
        &project_file,
    )?;
    storage.save_file(
        repository,
        serde_json::to_string_pretty(&version_data)?.as_bytes(),
        &version_folder,
    )?;
    Ok(())
}

pub fn get_version_data(
    storage: &StringStorage,
    repository: &Repository,
    project_folder: &str,
    project: &ProjectData,
) -> Result<(NPMTimes, DistTags, NPMVersions), InternalError> {
    let mut times = NPMTimes {
        created: format_time(project.created),
        modified: format_time(project.updated),
        times: Default::default(),
    };
    let dist_tags = DistTags {
        latest: project.versions.latest_version.clone(),
    };
    let mut npm_versions = HashMap::default();

    for version in &project.versions.versions {
        times
            .times
            .insert(version.version.clone(), format_time(version.time));
        let version_path = format!("{}/{}/package.json", project_folder, &version.version);
        let result = storage.get_file(repository, &version_path)?;
        if result.is_none() {
            warn!("{} not found", version_path);
            continue;
        }
        let version_data = result.unwrap();
        let version_data: Version = serde_json::from_slice(version_data.as_slice())?;
        npm_versions.insert(version.version.clone(), version_data);
    }

    Ok((times, dist_tags, npm_versions))
}

pub fn generate_get_response(
    storage: &StringStorage,
    repository: &Repository,
    project_folder: &str,
) -> Result<Option<GetResponse>, InternalError> {
    let option = get_project_data(storage, repository, project_folder.to_string())?;
    if option.is_none() {
        return Ok(None);
    }
    let project_data = option.unwrap();
    let (times, dist_tags, versions) =
        get_version_data(storage, repository, project_folder, &project_data)?;
    let version_path = format!("{}/{}/package.json", project_folder, &dist_tags.latest);
    let result = storage.get_file(repository, &version_path)?;
    if result.is_none() {
        warn!("{} not found", version_path);
        return Ok(None);
    }
    let version_data = result.unwrap();
    let version_data: Version = serde_json::from_slice(version_data.as_slice())?;
    Ok(Some(GetResponse {
        version_data,
        versions,
        times,
        dist_tags,
    }))
}

pub fn parse_project_to_directory(value: &str) -> String {
    value.replace('.', "/").replace(':', "/")
}

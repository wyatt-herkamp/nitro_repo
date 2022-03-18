use std::collections::HashMap;
use std::fs::{read_to_string, remove_file, File};
use std::io::Write;
use std::path::{Path};

use chrono::{DateTime, NaiveDateTime, Utc};

use crate::error::internal_error::InternalError;
use crate::repository::nitro::{NitroRepoVersions, ProjectData};
use crate::repository::types::VersionResponse;
use crate::repository::utils::get_versions;
use crate::utils::get_current_time;

use argon2::{Argon2, PasswordHash, PasswordVerifier};
use diesel::MysqlConnection;

use crate::repository::npm::models::LoginRequest;
use crate::system::action::get_user_by_username;

pub fn is_valid(
    username: &str,
    request: &LoginRequest,
    conn: &MysqlConnection,
) -> Result<bool, InternalError> {
    let result1 = get_user_by_username(username, conn)?;
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



pub fn update_project(project_folder:  &Path, version: String) -> Result<(), InternalError> {
    let buf = project_folder.join(".nitro.project.json");

    let mut project_data: ProjectData = if buf.exists() {
        let value = serde_json::from_str(&read_to_string(&buf)?).unwrap();
        remove_file(&buf)?;
        value
    } else {
        //TODO Pull NPM Data
        ProjectData {
            name: "".to_string(),
            description: "".to_string(),
            source: None,
            licence: None,
            versions: Default::default(),
            created: get_current_time(),
        }
    };
    project_data.versions.update_version(version);
    let mut file = File::create(&buf).unwrap();
    let string = serde_json::to_string_pretty(&project_data)?;
    let x1 = string.as_bytes();
    file.write_all(x1)?;
    Ok(())
}


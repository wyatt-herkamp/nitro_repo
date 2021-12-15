use std::collections::HashMap;
use std::fs::{DirEntry, File, read_dir, read_to_string, remove_file};
use std::io::Write;
use std::path::PathBuf;

use chrono::{DateTime, FixedOffset, NaiveDateTime, Utc};
use time::UtcOffset;

use crate::error::internal_error::InternalError;
use crate::repository::maven::models::DeployMetadata;
use crate::repository::nitro::{NitroMavenVersions, ProjectData};
use crate::repository::repository::VersionResponse;
use crate::utils::get_current_time;

pub fn get_versions(path: &PathBuf) -> NitroMavenVersions {
    let versions = path.join(".nitro.versions.json");
    return if versions.exists() {
        serde_json::from_str(&read_to_string(&versions).unwrap()).unwrap()
    } else {
        NitroMavenVersions {
            latest_version: "".to_string(),
            latest_release: "".to_string(),
            versions: vec![],
        }
    };
}

pub fn get_version(path: &PathBuf, version: String) -> Option<VersionResponse> {
    let buf = path.join(".nitro.versions.json");
    let versions_value: NitroMavenVersions = if buf.exists() {
        let value = serde_json::from_str(&read_to_string(&buf).unwrap()).unwrap();
        value
    } else {
        return None;
    };
    for x in versions_value.versions {
        if x.version.eq(&version) {
            return Some(VersionResponse {
                version: x,
                other: Default::default(),
            });
        }
    }
    return None;
}

/// Project format {groupID}:{artifactID}
pub fn parse_project_to_directory(value: &String) -> String {
    return value.replace(".", "/").replace(":", "/");
}

pub fn get_latest_version(path: &PathBuf, release: bool) -> Option<String> {
    let versions = path.join(".nitro.versions.json");
    return if versions.exists() {
        let option: NitroMavenVersions =
            serde_json::from_str(&read_to_string(&versions).unwrap()).unwrap();
        if release {
            Some(option.latest_release)
        } else {
            Some(option.latest_version)
        }
    } else {
        None
    };
}

fn get_artifacts(path: &PathBuf) -> Vec<String> {
    let dir = read_dir(path).unwrap();
    let mut values = Vec::new();
    for x in dir {
        let x1 = x.unwrap();
        if x1.file_type().unwrap().is_file() {
            let file_name = x1.file_name().to_str().unwrap().to_string();
            if file_name.ends_with(".sha1") || file_name.ends_with(".md5") {
                continue;
            }
            values.push(file_name);
        }
    }
    values
}

pub fn parse_maven_date_time(path: &str) -> Result<NaiveDateTime, InternalError> {
    let result = NaiveDateTime::parse_from_str(path, "%Y%m%d%H%M%S")?;
    return Ok(result);
}

#[cfg(test)]
mod tests {
    use crate::repository::maven::utils::parse_maven_date_time;

    #[test]
    fn parse_maven_date_time_test() {
        println!(
            "{}",
            parse_maven_date_time("20211201213303")
                .unwrap()
                .format("%Y-%m-%dT%H:%M:%S.%3fZ")
        );
    }
}

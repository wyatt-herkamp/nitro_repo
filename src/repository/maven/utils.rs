use std::collections::HashMap;
use std::fs::{DirEntry, File, read_dir, read_to_string, remove_file};
use std::io::Write;
use std::path::PathBuf;

use chrono::{DateTime, FixedOffset, NaiveDateTime, Utc};
use time::UtcOffset;

use crate::error::internal_error::InternalError;
use crate::repository::maven::models::{DeployMetadata, RepositoryListing};
use crate::repository::nitro::{NitroMavenVersions, ProjectData};
use crate::repository::repository::VersionResponse;
use crate::utils::get_current_time;

pub fn get_versions(path: &PathBuf) -> NitroMavenVersions {
    let versions = path.join(".nitro.versions.json");
    return
        if versions.exists() {
            serde_json::from_str(&read_to_string(&versions).unwrap()).unwrap()
        } else {
            NitroMavenVersions { latest_version: "".to_string(), latest_release: "".to_string(), versions: vec![] }
        };
}

pub fn get_version(path: &PathBuf, version: String) -> Option<VersionResponse> {
    let buf = path.join(".nitro.versions.json");
    let versions_value: NitroMavenVersions =
        if buf.exists() {
            let value = serde_json::from_str(&read_to_string(&buf).unwrap()).unwrap();
            value
        } else {
            return None;
        };
    for x in versions_value.versions {
        if x.version.eq(&version) {
            return Some(VersionResponse { version: x, other: Default::default() });
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
    return
        if versions.exists() {
            let option: NitroMavenVersions = serde_json::from_str(&read_to_string(&versions).unwrap()).unwrap();
            if release {
                Some(option.latest_release)
            } else {
                Some(option.latest_version)
            }
        } else {
            None
        };
}


pub fn update_versions(project_folder: &PathBuf, version: String) -> Result<(), InternalError> {
    let versions = project_folder.join(".nitro.versions.json");
    let mut versions_value: NitroMavenVersions =
        if versions.exists() {
            let value = serde_json::from_str(&read_to_string(&versions)?).unwrap();
            remove_file(&versions)?;
            value
        } else {
            NitroMavenVersions { latest_version: "".to_string(), latest_release: "".to_string(), versions: vec![] }
        };

    versions_value.update_version(version);
    let mut file = File::create(&versions).unwrap();
    let string = serde_json::to_string_pretty(&versions_value)?;
    let x1 = string.as_bytes();
    file.write_all(x1)?;
    return Ok(());
}

pub fn update_project_in_repositories(project: String, repo_location: PathBuf) -> Result<(), InternalError> {
    let buf = repo_location.join("repository.json");

    let mut repo_listing: RepositoryListing =
        if buf.exists() {
            let value = serde_json::from_str(&read_to_string(&buf)?).unwrap();
            value
        } else {
            RepositoryListing { values: vec![] }
        };

    if !repo_listing.add_value(project) && buf.exists() {
        remove_file(&buf)?;
    }
    let mut file = File::create(&buf).unwrap();
    let string = serde_json::to_string_pretty(&repo_listing)?;
    let x1 = string.as_bytes();
    file.write_all(x1)?;
    return Ok(());
}

pub fn update_project(project_folder: &PathBuf) -> Result<(), InternalError> {
    let buf = project_folder.join(".nitro.project.json");

    let mut repo_listing: ProjectData =
        if buf.exists() {
            let value = serde_json::from_str(&read_to_string(&buf)?).unwrap();
            remove_file(&buf);
            value
        } else {
            ProjectData { created: get_current_time() }
        };
    //TODO append updates
    let mut file = File::create(&buf).unwrap();
    let string = serde_json::to_string_pretty(&repo_listing)?;
    let x1 = string.as_bytes();
    file.write_all(x1)?;
    return Ok(());
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
        println!("{}", parse_maven_date_time("20211201213303").unwrap().format("%Y-%m-%dT%H:%M:%S.%3fZ"));
    }
}
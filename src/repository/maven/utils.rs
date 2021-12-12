use std::collections::HashMap;
use std::fs::{DirEntry, File, read_dir, read_to_string, remove_file};
use std::io::Write;
use std::path::PathBuf;

use chrono::{DateTime, FixedOffset, NaiveDateTime, Utc};
use time::UtcOffset;

use crate::error::internal_error::InternalError;
use crate::repository::maven::models::{DeployMetadata, NitroMavenVersions};
use crate::repository::repository::Version;

pub fn get_versions(path: &PathBuf) -> Vec<Version> {
    let maven_metadata = path.clone().join("maven-metadata.xml");
    let value = if maven_metadata.exists() {
        get_versions_generated(&maven_metadata)
    } else {
        get_versions_without_maven(path)
    };
    let mut versions = Vec::new();
    for x in value {
        versions.push(get_version(&path.join(x)))
    }
    versions
}

pub fn get_version(path: &PathBuf) -> Version {
    let mut other = HashMap::new();
    other.insert(
        "artifacts".to_string(),
        serde_json::to_value(get_artifacts(path)).unwrap(),
    );
    return Version {
        version: path.file_name().unwrap().to_str().unwrap().to_string(),
        other,
    };
}

fn get_versions_generated(path: &PathBuf) -> Vec<String> {
    let string = read_to_string(path).unwrap();
    let vec: DeployMetadata = serde_xml_rs::from_str(string.as_str()).unwrap();
    vec.versioning.versions.version
}

fn get_versions_without_maven(path: &PathBuf) -> Vec<String> {
    let dir = read_dir(path).unwrap();
    let mut values = Vec::new();
    for x in dir {
        let x1 = x.unwrap();
        if x1.file_type().unwrap().is_dir() {
            values.push(x1.file_name().to_str().unwrap().to_string());
        }
    }
    values
}

pub fn get_latest_version(path: &PathBuf, release: bool) -> String {
    let maven_metadata = path.join("maven-metadata.xml");

    if maven_metadata.exists() {
        get_latest_version_generated(&maven_metadata, release)
    } else {
        get_latest_versions_without_maven(path, release)
    }
}

fn get_latest_version_generated(path: &PathBuf, release: bool) -> String {
    let string = read_to_string(path).unwrap();
    let vec: DeployMetadata = serde_xml_rs::from_str(string.as_str()).unwrap();
    let versioning = vec.versioning;
    if release {
        if let Some(value) = versioning.release {
            return value;
        }
    }
    let versions = versioning.versions.version;
    for x in &versions {
        if release && (x.ends_with("SNAPSHOT") || x.contains("pr")) {
            continue;
        }
        return x.clone();
    }
    return versions.first().unwrap_or(&String::new()).clone();
}

fn get_latest_versions_without_maven(path: &PathBuf, release: bool) -> String {
    let dir = read_dir(path).unwrap();
    let vec = dir.collect::<Vec<Result<DirEntry, std::io::Error>>>();
    let mut values = vec
        .iter()
        .map(|e| e.as_ref().unwrap())
        .collect::<Vec<&DirEntry>>();
    values.sort_by(|a, b| {
        a.metadata()
            .unwrap()
            .created()
            .unwrap()
            .cmp(&b.metadata().unwrap().created().unwrap())
    });

    let mut value = None;
    for x in values {
        if x.file_type().unwrap().is_dir() {
            let string = x.file_name().to_str().unwrap().to_string();
            if value.is_none() {
                value = Some(string.clone());
            }
            if release && (string.ends_with("SNAPSHOT") || string.contains("pr")) {
                continue;
            }
            return string;
        }
    }
    value.unwrap_or("".to_string())
}

pub fn update_versions(project_folder: &PathBuf, version: String) -> Result<(), InternalError> {
    let versions = project_folder.join(".nitro.versions.json");
    let mut versions_value: NitroMavenVersions =
        if versions.exists() {
            let value = serde_json::from_str(&read_to_string(&versions)?).unwrap();
            remove_file(&versions)?;
            value
        } else {
            NitroMavenVersions { version: vec![] }
        };

    versions_value.update_version(version);
    let mut file = File::create(&versions).unwrap();
    let string = serde_json::to_string(&versions)?;
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
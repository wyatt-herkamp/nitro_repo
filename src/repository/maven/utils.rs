use crate::repository::repository::Version;
use std::fs::{read_dir, read_to_string, DirEntry};
use std::path::PathBuf;

use crate::repository::maven::models::DeployMetadata;

pub fn get_versions(path: &PathBuf) -> Vec<Version> {
    let maven_metadata = path.clone().join("maven-metadata.xml");
    let value = if maven_metadata.exists() {
        get_versions_generated(&maven_metadata)
    } else {
        get_versions_without_maven(path)
    };
    let mut versions = Vec::new();
    for x in value {
        versions.push(Version {
            version: x.clone(),
            artifacts: get_artifacts(path.clone().join(x)),
        })
    }
    return versions;
}

fn get_versions_generated(path: &PathBuf) -> Vec<String> {
    let string = read_to_string(path).unwrap();
    let vec: DeployMetadata = serde_xml_rs::from_str(string.as_str()).unwrap();
    return vec.versioning.versions.version;
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
    return values;
}

pub fn get_latest_version(path: &PathBuf, release: bool) -> String {
    let maven_metadata = path.join("maven-metadata.xml");
    let value = if maven_metadata.exists() {
        get_latest_version_generated(&maven_metadata, release)
    } else {
        get_latest_versions_without_maven(path, release)
    };
    return value;
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
        if release {
            if x.ends_with("SNAPSHOT") || x.contains("pr") {
                continue;
            }
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
            if release {
                if string.ends_with("SNAPSHOT") || string.contains("pr") {
                    continue;
                }
            }
            return string;
        }
    }
    return value.unwrap_or("".to_string());
}

fn get_artifacts(path: PathBuf) -> Vec<String> {
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
    return values;
}

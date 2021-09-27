use std::path::PathBuf;
use std::fs::{read_dir, read_to_string};
use crate::repository::repository::Version;
use rust_embed::utils::read_file_from_fs;
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
            artifacts: get_artifacts(path.clone().join(x.clone())),
        })
    }
    return versions;
}

fn get_versions_generated(path: &PathBuf) -> Vec<String> {
    let string = read_to_string(path).unwrap();
    let vec: DeployMetadata = serde_xml_rs::from_str(string.as_str()).unwrap();
    println!("{}", &vec.artifact_id);
    println!("{:?}", &vec.versioning.release);
    return vec.versioning.versions.version.clone();
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
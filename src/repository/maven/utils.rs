use std::fs::{read_dir, read_to_string};
use std::path::PathBuf;

use chrono::NaiveDateTime;

use crate::error::internal_error::InternalError;
use crate::repository::nitro::NitroMavenVersions;
use crate::repository::repository::VersionResponse;
use crate::repository::utils::get_versions;

pub fn get_version(path: &PathBuf, version: String) -> Option<VersionResponse> {
    let versions_value = get_versions(path);
    return get_version_by_data(&versions_value, version);
}

pub fn get_version_by_data(versions_value: &NitroMavenVersions, version: String) -> Option<VersionResponse> {
    for x in &versions_value.versions {
        if x.version.eq(&version) {
            return Some(VersionResponse {
                version: x.clone(),
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

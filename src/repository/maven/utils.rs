use std::fs::{read_dir, read_to_string, remove_file, File};
use std::io::Write;
use std::path::Path;

use chrono::{NaiveDateTime};

use crate::error::internal_error::InternalError;
use crate::repository::maven::models::Pom;
use crate::repository::nitro::{NitroMavenVersions, ProjectData};
use crate::repository::types::VersionResponse;
use crate::repository::utils::get_versions;
use crate::utils::get_current_time;


pub fn get_version(path: &Path, version: String) -> Option<VersionResponse> {
    let versions_value = get_versions(path);
    get_version_by_data(&versions_value, version)
}

pub fn get_version_by_data(
    versions_value: &NitroMavenVersions,
    version: String,
) -> Option<VersionResponse> {
    for x in &versions_value.versions {
        if x.version.eq(&version) {
            return Some(VersionResponse {
                version: x.clone(),
                other: Default::default(),
            });
        }
    }
    None
}

/// Project format {groupID}:{artifactID}
pub fn parse_project_to_directory(value: &str) -> String {
    value.replace('.', "/").replace(':', "/")
}

#[allow(dead_code)]
fn get_artifacts(path: &Path) -> Vec<String> {
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

#[allow(dead_code)]
pub fn parse_maven_date_time(path: &str) -> Result<NaiveDateTime, InternalError> {
    let result = NaiveDateTime::parse_from_str(path, "%Y%m%d%H%M%S")?;
    Ok(result)
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


pub fn update_project(
    project_folder: &Path,
    version: String,
    pom: Pom,
) -> Result<(), InternalError> {
    let buf = project_folder.join(".nitro.project.json");

    let mut project_data: ProjectData = if buf.exists() {
        let value = serde_json::from_str(&read_to_string(&buf)?).unwrap();
        remove_file(&buf)?;
        value
    } else {
        ProjectData {
            name: format!("{}:{}", &pom.group_id, &pom.artifact_id),
            description: "".to_string(),
            source: None,
            licence: None,
            versions: Default::default(),
            created: get_current_time(),
        }
    };
    project_data.description = pom.description.unwrap_or_else(|| "".to_string());
    project_data.versions.update_version(version);
    let mut file = File::create(&buf).unwrap();
    let string = serde_json::to_string_pretty(&project_data)?;
    let x1 = string.as_bytes();
    file.write_all(x1)?;
    Ok(())
}

use std::fs::{read_dir};
use std::path::Path;

use crate::constants::PROJECT_FILE;
use chrono::NaiveDateTime;
use log::trace;

use crate::error::internal_error::{InternalError, NResult};
use crate::repository::maven::models::Pom;
use crate::repository::models::Repository;
use crate::repository::nitro::{NitroRepoVersions, ProjectData};
use crate::repository::types::VersionResponse;
use crate::repository::utils::get_versions;
use crate::storage::models::StringStorage;
use crate::utils::get_current_time;


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
    storage: &StringStorage,
    repository: &Repository,
    project_folder: &str,
    version: String,
    pom: Pom,
) -> Result<(), InternalError> {
    let project_file = format!("{}/{}", project_folder, PROJECT_FILE);
    trace!("Project File Location {}", project_file);
    let option = storage.get_file(repository, &project_file)?;
    let mut project_data: ProjectData = if let Some(data) = option {
        let string = String::from_utf8(data)?;
        let value = serde_json::from_str(&string)?;
        storage.delete_file(repository, &project_file)?;
        value
    } else {
        ProjectData {
            name: format!("{}:{}", &pom.group_id, &pom.artifact_id),
            description: "".to_string(),
            source: None,
            licence: None,
            versions: Default::default(),
            created: get_current_time(),
            updated: get_current_time()
        }
    };
    project_data.description = pom.description.unwrap_or_else(|| "".to_string());
    project_data.versions.update_version(version);
    storage.save_file(
        repository,
        serde_json::to_string_pretty(&project_data)?.as_bytes(),
        &project_file,
    )?;
    Ok(())
}

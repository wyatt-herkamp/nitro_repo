use std::collections::HashMap;

use log::warn;

use chrono::{DateTime, NaiveDateTime, Utc};

use crate::repository::nitro::{NitroRepoVersions, ProjectData};

use crate::error::internal_error::InternalError;
use crate::repository::data::RepositoryConfig;
use crate::repository::nitro::utils::get_project_data;

use crate::repository::npm::models::{DistTags, GetResponse, NPMTimes, NPMVersions, Version};
use crate::storage::models::Storage;
use crate::storage::DynamicStorage;

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

pub async fn get_version_data(
    storage: &DynamicStorage,
    repository: &RepositoryConfig,
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
        let result = storage.get_file(repository, &version_path).await?;
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

pub async fn generate_get_response(
    storage: &DynamicStorage,
    repository: &RepositoryConfig,
    project_folder: &str,
) -> Result<Option<GetResponse>, InternalError> {
    let option = get_project_data(storage, repository, project_folder).await?;
    if option.is_none() {
        return Ok(None);
    }
    let project_data = option.unwrap();
    let (times, dist_tags, versions) =
        get_version_data(storage, repository, project_folder, &project_data).await?;
    let version_path = format!("{}/{}/package.json", project_folder, &dist_tags.latest);
    let result = storage.get_file(repository, &version_path).await?;
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

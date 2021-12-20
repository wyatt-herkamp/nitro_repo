use std::collections::HashMap;
use std::fs::{create_dir_all, File};
use std::path::PathBuf;

use chrono::{DateTime, NaiveDateTime, Utc};

use crate::error::internal_error::InternalError;
use crate::repository::models::Repository;
use crate::repository::nitro::NitroMavenVersions;
use crate::repository::repository::VersionResponse;
use crate::repository::utils::get_versions;
use crate::storage::models::Storage;
use crate::utils::get_storage_location;

static NPM_TIME_FORMAT: &'static str = "%Y-%m-%dT%H:%M:%S.%3fZ";

impl From<NitroMavenVersions> for HashMap<String, String> {
    fn from(value: NitroMavenVersions) -> Self {
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

pub fn get_version(path: &PathBuf, version: String) -> Option<VersionResponse> {
    let versions_value = get_versions(path);
    return get_version_by_data(&versions_value, version);
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
    return None;
}

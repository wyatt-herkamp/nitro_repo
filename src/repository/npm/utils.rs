use std::collections::HashMap;
use std::fs::{create_dir_all, File};
use std::path::PathBuf;

use chrono::{DateTime, NaiveDateTime, Utc};

use crate::error::internal_error::InternalError;
use crate::repository::models::Repository;
use crate::repository::nitro::NitroMavenVersions;
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

pub fn get_time_file<S: Into<String>>(storage: &Storage, repo: &Repository, id: S) -> PathBuf {
    let string = id.into();
    let buf = get_storage_location()
        .join("storages")
        .join(&storage.name)
        .join(&repo.name)
        .join(string.replace("%2f", "/"));
    if !buf.exists() {
        create_dir_all(&buf);
    }
    return buf.join("times.json");
}

pub fn read_time_file<S: Into<String>>(
    storage: &Storage,
    repo: &Repository,
    id: S,
) -> Result<HashMap<String, String>, InternalError> {
    let times_json = get_time_file(&storage, &repo, id);
    let times_map: HashMap<String, String> = serde_json::from_reader(File::open(&times_json)?)?;
    return Ok(times_map);
}

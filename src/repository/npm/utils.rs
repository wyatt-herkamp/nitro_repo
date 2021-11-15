use std::collections::HashMap;
use std::fs::File;
use std::path::PathBuf;

use crate::error::internal_error::InternalError;
use crate::repository::models::Repository;
use crate::storage::models::Storage;
use crate::utils::get_storage_location;

pub fn get_time_file<S: Into<String>>(storage: &Storage, repo: &Repository, id: S) -> PathBuf {
    return get_storage_location()
        .join("storages")
        .join(&storage.name)
        .join(&repo.name).join(id.into()).join("times.json");
}

pub fn read_time_file<S: Into<String>>(storage: &Storage, repo: &Repository, id: S) -> Result<HashMap<String, String>, InternalError> {
    let times_json = get_time_file(&storage, &repo, id);
    let times_map: HashMap<String, String> = serde_json::from_reader(File::open(&times_json)?)?;
    return Ok(times_map);
}
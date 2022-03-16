use std::collections::HashMap;
use std::fmt::Debug;
use std::fs;
use std::fs::{OpenOptions, read_to_string};
use std::io::Write;
use std::path::Path;

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::error::internal_error::InternalError;
use crate::StringMap;

pub static STORAGE_FILE: &str = "storages.json";
pub static STORAGE_FILE_BAK: &str = "storages.json.bak";

pub fn load_storages() -> anyhow::Result<Storages> {
    let path = Path::new(STORAGE_FILE);
    if !path.exists() {
        return Ok(HashMap::new());
    }
    let string = read_to_string(&path)?;
    let result: Storages = toml::from_str(&string)?;
    return Ok(result);
}

pub fn save_storages(storages: &Storages) -> Result<(), InternalError> {
    let result = serde_json::to_string(&storages)?;
    let path = Path::new(STORAGE_FILE);
    let bak = Path::new(STORAGE_FILE_BAK);
    if bak.exists() {
        fs::remove_file(bak)?;
    }
    if path.exists() {
        fs::rename(path, bak)?;
    }
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .open(path)?;
    file.write_all(result.as_bytes())?;
    return Ok(());
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LocationType {
    LocalStorage,
}

pub type Storages = HashMap<Uuid, Storage<StringMap>>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Storage<T> {
    pub public_name: String,
    pub name: String,
    pub created: i64,
    pub location_type: LocationType,
    #[serde(flatten)]
    pub location: T,
}


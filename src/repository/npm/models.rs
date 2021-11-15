use std::collections::HashMap;

use chrono::{TimeZone, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginRequest {
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginResponse {
    pub ok: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Attachment {
    pub content_type: String,
    pub data: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PublishRequest {
    pub name: String,
    pub _attachments: HashMap<String, Value>,
    pub versions: HashMap<String, Version>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Version {
    pub version: String,
    pub name: String,
    #[serde(flatten)]
    pub other: HashMap<String, Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetResponse {
    pub id: String,
    pub name: String,
    pub versions: HashMap<String, Version>,
    pub times: HashMap<String, String>,
    #[serde(flatten)]
    pub other: HashMap<String, Value>,
}

pub fn get_latest_version(map: &HashMap<String, String>) -> String {
    let value = map.values().cloned().collect::<Vec<String>>();
    let mut times = Vec::new();
    for x in value {
        println!("{}", &x);
        let result = Utc.datetime_from_str(&x, "%Y-%m-%dT%H:%M:%S.%3fZ");
        if let Err(err) = result {
            println!("{}", err.to_string());
            continue;
        }
        times.push(result.unwrap());
    }
    println!("{}", times.len());
    times.sort();
    times.reverse();
    let first_time = times
        .first()
        .unwrap()
        .format("%Y-%m-%dT%H:%M:%S.%3fZ")
        .to_string();
    for (version, time) in map {
        if time.eq(&first_time) {
            return version.clone();
        }
    }
    return "unknown".to_string();
}

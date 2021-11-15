use std::collections::HashMap;
use std::io::Bytes;
use std::iter::Map;

use actix::ActorStreamExt;
use chrono::{DateTime, FixedOffset, TimeZone, Utc};
use futures_util::SinkExt;
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
    let first_time = times.first().unwrap().format("%Y-%m-%dT%H:%M:%S.%3fZ").to_string();
    for (version, time) in map {
        if time.eq(&first_time) {
            return version.clone();
        }
    }
    return "unknown".to_string();
}

#[cfg(test)]
mod Test {
    use std::collections::HashMap;

    use crate::repository::npm::models::get_latest_version;

    #[test]
    fn it_works() {
        let mut value = HashMap::<String, String>::new();
        value.insert("created".to_string(), "2021-11-15T17:54:28.372Z".to_string());
        value.insert("0.1.0".to_string(), "2021-11-15T17:54:28.372Z".to_string());
        value.insert("0.1.1".to_string(), "2021-11-15T18:54:28.372Z".to_string());
        value.insert("0.1.2".to_string(), "2021-12-15T19:54:28.372Z".to_string());
        assert_eq!(get_latest_version(&value), "0.1.2".to_string());
    }
}
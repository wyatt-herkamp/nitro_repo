use crate::apierror::APIError;



use chrono::{DateTime, Duration, Local};
use diesel::MysqlConnection;


use serde::{Deserialize, Serialize};
use std::ops::Add;



use crate::settings::action::get_setting;





use rust_embed::RustEmbed;
use std::fs::read;
use std::path::{Path};

#[derive(RustEmbed)]
#[folder = "$CARGO_MANIFEST_DIR/resources"]
pub struct Resources;

impl Resources {
    pub fn file_get(file: &str) -> Vec<u8> {
        let buf = Path::new("resources").join(file);
        if buf.exists() {
            return read(buf).unwrap();
        } else {
            return Resources::get(file).unwrap().data.to_vec();
        }
    }
    pub fn file_get_string(file: &str) -> String {
        let vec = Resources::file_get(file);
        return String::from_utf8(vec).unwrap();
    }
}

pub fn installed(conn: &MysqlConnection) -> Result<(), APIError> {
    let option = get_setting("INSTALLED", &conn)?;
    if option.is_none() {
        return Err(APIError::UnInstalled);
    }

    return Ok(());
}

pub fn get_current_time() -> i64 {
    Local::now().timestamp_millis()
}

pub fn get_current_date_time() -> String {
    let local: DateTime<Local> = Local::now();
    let format = local.format("%B %d %Y %H:%M");
    format.to_string()
}

pub fn default_expiration() -> i64 {
    let time = Local::now();
    time.add(Duration::days(30)).timestamp_millis()
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct EmailChangeRequest {
    pub email_username: Option<String>,
    pub email_password: Option<String>,
    pub email_host: Option<String>,
    pub encryption: Option<String>,
    pub from: Option<String>,
    pub port: Option<i64>,
}

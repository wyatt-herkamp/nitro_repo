use std::borrow::Cow;
use std::fs::OpenOptions;
use std::io::Read;
use std::ops::Add;
use std::path::Path;

use actix_web::http::header::HeaderMap;
use chrono::{DateTime, Duration, Local};
use nitro_log::config::Config;
use nitro_log::{LoggerBuilders, NitroLogger};
use rust_embed::RustEmbed;

use crate::error::internal_error::InternalError;
use crate::settings::models::Mode;

pub fn load_logger<T: AsRef<Mode>>(logger: T) {
    let file = match logger.as_ref() {
        Mode::Debug => "log-debug.json",
        Mode::Release => "log-release.json",
        Mode::Install => "log-install.json",
    };
    let config: Config = serde_json::from_slice(
        Resources::file_get(file)
            .as_ref()
            .expect("Unable to load the logger!"),
    )
    .unwrap();
    NitroLogger::load(config, LoggerBuilders::default()).unwrap();
}
#[derive(RustEmbed)]
#[folder = "$CARGO_MANIFEST_DIR/resources"]
pub struct Resources;

impl Resources {
    pub fn file_get<'a>(file: &str) -> Result<Cow<'a, [u8]>, InternalError> {
        let buf = Path::new("resources").join(file);
        if buf.exists() {
            let mut buffer = Vec::new();
            OpenOptions::new()
                .read(true)
                .open(buf)?
                .read_to_end(&mut buffer)?;
            Ok(Cow::Owned(buffer))
        } else {
            Ok(Resources::get(file).unwrap().data)
        }
    }
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

pub fn get_accept(header_map: &HeaderMap) -> Result<Option<String>, InternalError> {
    let option = header_map.get("accept");
    if option.is_none() {
        return Ok(None);
    }
    let x = option.unwrap().to_str();
    if x.is_err() {}
    let header = x.unwrap().to_string();
    Ok(Some(header))
}

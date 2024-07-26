use std::borrow::Cow;
use std::fs::OpenOptions;
use std::io::Read;
use std::ops::Add;
use std::path::Path;

use actix_web::http::header::HeaderMap;
use chrono::{DateTime, Duration, FixedOffset, Local};
use rust_embed::RustEmbed;
use tracing::error;
use tracing_subscriber::Layer;

use crate::error::internal_error::InternalError;

#[derive(RustEmbed)]
#[folder = "$CARGO_MANIFEST_DIR/resources"]
pub struct Resources;

impl Resources {
    ///  Gets the file from the resources file if it exists or defaults to the embedded file.
    ///
    /// # Arguments
    ///
    /// * `file`: Relative path to the file.
    ///
    /// returns: Result<Cow<[u8]>, InternalError>
    /// Errors are returned if the IO operation fails.
    /// # Panics
    /// If the embedded resource is not found.
    /// This should never happen.
    /// If it does, it is a bug.
    /// Please report it.
    pub fn file_get(file: &str) -> Result<Cow<'_, [u8]>, InternalError> {
        let buf = Path::new("resources").join(file);
        if buf.exists() {
            let mut file = match OpenOptions::new().read(true).open(buf) {
                Ok(ok) => ok,
                Err(err) => {
                    error!("Unable to open the {file:?}: {}", err);
                    return Err(InternalError::from(err));
                }
            };
            let mut buffer = Vec::with_capacity(file.metadata()?.len() as usize);
            file.read_to_end(&mut buffer)?;
            Ok(Cow::Owned(buffer))
        } else {
            Ok(Resources::get(file)
                .expect("Embedded Resource was not found")
                .data)
        }
    }
}
pub fn get_current_date_time_struct() -> DateTime<FixedOffset> {
    let local: DateTime<Local> = Local::now();
    local.with_timezone(local.offset())
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

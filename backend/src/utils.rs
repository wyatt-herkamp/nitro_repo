use std::borrow::Cow;
use std::fs::OpenOptions;
use std::io::Read;
use std::ops::Add;
use std::path::Path;

use crate::authentication;
use actix_web::http::header::HeaderMap;
use chrono::{DateTime, Duration, Local};
use nitro_log::config::Config;
use nitro_log::{LoggerBuilders, NitroLogger};
use rust_embed::RustEmbed;
use sea_orm::{DatabaseConnection, DbErr, Schema};

use crate::error::internal_error::InternalError;
use crate::settings::models::Mode;
use crate::system::user::UserEntity;
use sea_orm::ConnectionTrait;
pub async fn run_database_setup(database: &mut DatabaseConnection) -> Result<(), DbErr> {
    let schema = Schema::new(database.get_database_backend());
    let users = schema.create_table_from_entity(UserEntity);
    database
        .execute(database.get_database_backend().build(&users))
        .await?;
    let tokens = schema.create_table_from_entity(authentication::auth_token::AuthTokenEntity);
    database
        .execute(database.get_database_backend().build(&tokens))
        .await?;
    Ok(())
}
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

pub mod base64_utils{
    use base64::{DecodeError, Engine};
    use base64::engine::general_purpose::STANDARD;

    pub fn decode(input: impl AsRef<[u8]>) -> Result<Vec<u8>, DecodeError>{
        STANDARD.decode(input)
    }
    pub fn decode_as_string(input: impl AsRef<[u8]>) ->Result<String, actix_web::Error>{
        let decoded = decode(input).map_err(|e| actix_web::error::ErrorBadRequest(e))?;
        String::from_utf8(decoded).map_err(|x| actix_web::error::ErrorBadRequest(x))
    }
    pub fn encode(input: impl AsRef<[u8]>) -> String{
        STANDARD.encode(input)
    }
    pub fn encode_basic_header(username: impl AsRef<str>, password: impl AsRef<str>)->String{
        STANDARD.encode(format!("{}:{}", username.as_ref(), password.as_ref()))
    }
}
use std::ops::Add;

use actix_web::http::header::HeaderMap;
use chrono::{DateTime, Duration, FixedOffset, Local};
use sea_orm::{ConnectionTrait, DatabaseConnection, DbErr, Schema};

use crate::{authentication, error::internal_error::InternalError, system::user::UserEntity};
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
    let Some(value) = header_map.get("accept") else {
        return Ok(None);
    };
    value
        .to_str()
        .map(|x| Some(x.to_owned()))
        .map_err(InternalError::from)
}

pub mod base64_utils {
    use base64::{engine::general_purpose::STANDARD, DecodeError, Engine};

    pub fn decode(input: impl AsRef<[u8]>) -> Result<Vec<u8>, DecodeError> {
        STANDARD.decode(input)
    }
    pub fn decode_as_string(input: impl AsRef<[u8]>) -> Result<String, actix_web::Error> {
        let decoded = decode(input).map_err(|e| actix_web::error::ErrorBadRequest(e))?;
        String::from_utf8(decoded).map_err(|x| actix_web::error::ErrorBadRequest(x))
    }
    pub fn encode(input: impl AsRef<[u8]>) -> String {
        STANDARD.encode(input)
    }
    pub fn encode_basic_header(username: impl AsRef<str>, password: impl AsRef<str>) -> String {
        STANDARD.encode(format!("{}:{}", username.as_ref(), password.as_ref()))
    }
}

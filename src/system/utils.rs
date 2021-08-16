use crate::system;
use actix_web::http::HeaderMap;
use diesel::MysqlConnection;
use crate::system::models::User;
use crate::siteerror::SiteError;
use actix_web::{HttpRequest, HttpMessage};

pub fn get_user_by_cookie(
    http: &HttpRequest,
    conn: &MysqlConnection,
) -> Result<Option<User>, SiteError> {
    let option = http.cookie("session");
    if option.is_none() {
        return Ok(None);
    }
    let x = option.as_ref().unwrap().value().clone();

    let result = system::action::get_user_from_auth_token(x.to_string(), conn)?;
    return Ok(result);
}

pub fn get_user_by_header(
    header_map: &HeaderMap,
    conn: &MysqlConnection,
) -> Result<Option<User>, SiteError> {
    let option = header_map.get("Authorization");
    if option.is_none() {
        return Ok(None);
    }
    let x = option.unwrap().to_str();
    if x.is_err() {}
    let header = x.unwrap().to_string();

    let split = header.split(" ").collect::<Vec<&str>>();
    let option = split.get(0);
    if option.is_none() {
        return Ok(None);
    }
    let value = split.get(1);
    if value.is_none() {
        return Ok(None);
    }
    let value = value.unwrap().to_string();
    let key = option.unwrap().to_string();
    if key.eq("Bearer") {
        let result = system::action::get_user_from_auth_token(value, conn)?;
        return Ok(result);
    }
    Ok(None)
}

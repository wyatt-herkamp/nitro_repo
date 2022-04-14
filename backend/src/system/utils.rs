use actix_web::http::header::HeaderMap;
use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use diesel::MysqlConnection;
use rand::distributions::Alphanumeric;
use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::error::internal_error::InternalError;
use crate::repository::models::{Repository};
use crate::repository::settings::security::Visibility;
use crate::system;
use crate::system::action::{get_session_token, get_user_by_username};
use crate::system::models::User;
use crate::system::permissions::{can_deploy, can_read};

pub fn get_user_by_header(
    header_map: &HeaderMap,
    conn: &MysqlConnection,
) -> Result<Option<User>, InternalError> {
    let option = header_map.get("Authorization");
    if option.is_none() {
        return Ok(None);
    }
    let x = option.unwrap().to_str();
    if x.is_err() {}
    let header = x.unwrap().to_string();

    let split = header.split(' ').collect::<Vec<&str>>();

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
        let result = system::action::get_user_from_session_token(&value, conn)?;
        return Ok(result);
    } else if key.eq("Basic") {
        let result = base64::decode(value)?;
        let string = String::from_utf8(result)?;
        let split = string.split(':').collect::<Vec<&str>>();

        if !split.len().eq(&2) {
            return Ok(None);
        }
        let result1 = get_user_by_username(split.get(0).unwrap(), conn)?;
        if result1.is_none() {
            return Ok(None);
        }
        let argon2 = Argon2::default();
        let user = result1.unwrap();
        let parsed_hash = PasswordHash::new(user.password.as_str())?;
        if argon2
            .verify_password(split.get(1).unwrap().as_bytes(), &parsed_hash)
            .is_err()
        {
            return Ok(None);
        }
        return Ok(Some(user));
    }
    Ok(None)
}

pub fn can_deploy_basic_auth(
    header_map: &HeaderMap,
    repo: &Repository,
    conn: &MysqlConnection,
) -> Result<(bool, Option<User>), InternalError> {
    let option = get_user_by_header(header_map, conn)?;
    if option.is_none() {
        return Ok((false, None));
    }
    let user = option.unwrap();
    Ok((can_deploy(&user.permissions, repo)?, Some(user)))
}

pub fn can_read_basic_auth(
    header_map: &HeaderMap,
    repo: &Repository,
    conn: &MysqlConnection,
) -> Result<(bool, Option<User>), InternalError> {
    match repo.security.visibility {
        Visibility::Public => Ok((true, None)),
        Visibility::Private => {
            let option = get_user_by_header(header_map, conn)?;
            if option.is_none() {
                return Ok((false, None));
            }
            let user = option.unwrap();
            Ok((can_read(&user.permissions, repo)?, Some(user)))
        }
        Visibility::Hidden => Ok((true, None)),
    }
}


#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct NewUser {
    pub name: String,
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ModifyUser {
    pub name: String,
    pub email: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct NewPassword {
    pub password: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum NewUserError {
    UsernameAlreadyExists,
    UsernameMissing,
    EmailAlreadyExists,
    EmailMissing,
    PasswordDoesNotMatch,
    PasswordMissing,
}

pub fn hash(password: String) -> Result<String, InternalError> {
    let salt = SaltString::generate(&mut OsRng);

    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(password.as_bytes(), salt.as_ref())?
        .to_string();
    Ok(password_hash)
}

pub fn generate_session_token(connection: &MysqlConnection) -> Result<String, InternalError> {
    loop {
        let x: String = OsRng
            .sample_iter(&Alphanumeric)
            .take(128)
            .map(char::from)
            .collect();
        let result = get_session_token(&x, connection)?;
        if result.is_none() {
            return Ok(x);
        }
    }
}

pub fn generate_auth_token() -> String {
    let x: String = OsRng
        .sample_iter(&Alphanumeric)
        .take(64)
        .map(char::from)
        .collect();
    return format!("ntr_{}", x);
}

use actix_web::http::HeaderMap;
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::SaltString;
use diesel::insertable::ColumnInsertValue::Default;
use diesel::MysqlConnection;
use rand::distributions::Alphanumeric;
use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::error::internal_error::InternalError;
use crate::error::response::already_exists;
use crate::repository::models::{Repository, Visibility};
use crate::system;
use crate::system::action::{add_new_user, get_session_token, get_user_by_username};
use crate::system::models::{User, UserPermissions};
use crate::system::utils::NewUserError::{
    EmailAlreadyExists, PasswordDoesNotMatch, UsernameAlreadyExists,
};
use crate::utils::get_current_time;

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
    }
    Ok(None)
}

pub fn can_deploy_basic_auth(
    header_map: &HeaderMap,
    repo: &Repository,
    conn: &MysqlConnection,
) -> Result<bool, InternalError> {
    let option = header_map.get("Authorization");
    if option.is_none() {
        return Ok(false);
    }
    let x = option.unwrap().to_str();
    if x.is_err() {}
    let header = x.unwrap().to_string();
    let split = header.split(' ').collect::<Vec<&str>>();
    let option = split.get(0);
    if option.is_none() {
        return Ok(false);
    }
    let value = split.get(1);
    if value.is_none() {
        return Ok(false);
    }
    let value = value.unwrap().to_string();
    let key = option.unwrap().to_string();
    if key.eq("Basic") {
        return is_authed_deploy(value, repo, conn);
    } else if key.eq("Bearer") {
        let result = system::action::get_user_from_session_token(&value, &conn)?;
        if result.is_none() {
            return Ok(false);
        }
        return user_has_deploy_access(&result.unwrap(), repo);
    }
    Ok(false)
}

pub fn can_read_basic_auth(
    header_map: &HeaderMap,
    repo: &Repository,
    conn: &MysqlConnection,
) -> Result<bool, InternalError> {
    match repo.security.visibility {
        Visibility::Public => Ok(true),
        Visibility::Private => {
            let option = header_map.get("Authorization");
            if option.is_none() {
                return Ok(false);
            }
            let x = option.unwrap().to_str();
            if x.is_err() {}
            let header = x.unwrap().to_string();

            let split = header.split(' ').collect::<Vec<&str>>();
            let option = split.get(0);
            if option.is_none() {
                return Ok(false);
            }
            let value = split.get(1);
            if value.is_none() {
                return Ok(false);
            }
            let value = value.unwrap().to_string();
            let key = option.unwrap().to_string();
            if key.eq("Basic") {
                return is_authed_read(value, repo, conn);
            } else if key.eq("Bearer") {
                let result = system::action::get_user_from_session_token(&value, &conn)?;
                if result.is_none() {
                    return Ok(false);
                }
                return Ok(true);
            }
            Ok(false)
        }
        Visibility::Hidden => Ok(true),
    }
}

pub fn is_authed_deploy(
    user: String,
    repo: &Repository,
    conn: &MysqlConnection,
) -> Result<bool, InternalError> {
    let result = base64::decode(user)?;
    let string = String::from_utf8(result)?;
    let split = string.split(':').collect::<Vec<&str>>();

    if !split.len().eq(&2) {
        return Ok(false);
    }
    let result1 = get_user_by_username(&split.get(0).unwrap().to_string(), conn)?;
    if result1.is_none() {
        return Ok(false);
    }
    let argon2 = Argon2::default();
    let user = result1.unwrap();
    let parsed_hash = PasswordHash::new(user.password.as_str())?;
    if argon2
        .verify_password(split.get(1).unwrap().as_bytes(), &parsed_hash)
        .is_err()
    {
        return Ok(false);
    }
    return user_has_deploy_access(&user, repo);
}

pub fn is_authed_read(
    user: String,
    _repo: &Repository,
    conn: &MysqlConnection,
) -> Result<bool, InternalError> {
    let result = base64::decode(user)?;
    let string = String::from_utf8(result)?;
    let split = string.split(':').collect::<Vec<&str>>();

    if !split.len().eq(&2) {
        return Ok(false);
    }
    let result1 = get_user_by_username(&split.get(0).unwrap().to_string(), conn)?;
    if result1.is_none() {
        return Ok(false);
    }
    let argon2 = Argon2::default();
    let user = result1.unwrap();
    let parsed_hash = PasswordHash::new(user.password.as_str())?;
    if argon2
        .verify_password(split.get(1).unwrap().as_bytes(), &parsed_hash)
        .is_err()
    {
        return Ok(false);
    }
    return Ok(true);
}

pub fn user_has_deploy_access(user: &User, repo: &Repository) -> Result<bool, InternalError> {
    if !user.permissions.admin {
        if !repo.security.deployers.is_empty() {
            return Ok(user.permissions.deployer.clone());
        } else {
            return Ok(repo.security.deployers.contains(&user.id));
        }
    }
    return Ok(true);
}

/// TODO call this method for reading
pub fn user_has_read_access(user: &User, repo: &Repository) -> Result<bool, InternalError> {
    return if !repo.security.readers.is_empty() {
        Ok(true)
    } else {
        Ok(repo.security.readers.contains(&user.id))
    };
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct NewUser {
    pub name: String,
    pub username: String,
    pub email: String,
    pub password: String
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

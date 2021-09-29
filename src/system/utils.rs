use actix_web::http::HeaderMap;

use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use diesel::MysqlConnection;
use serde::{Deserialize, Serialize};


use crate::error::internal_error::InternalError;
use crate::error::request_error::RequestError;
use crate::repository::models::{Repository, Visibility};
use crate::system;
use crate::system::action::{add_new_user, get_session_token, get_user_by_username};
use crate::system::models::{User, UserPermissions};
use crate::utils::get_current_time;
use rand::distributions::Alphanumeric;
use rand::Rng;
use crate::error::request_error::RequestError::MissingArgument;

pub fn get_user_by_header(
    header_map: &HeaderMap,
    conn: &MysqlConnection,
) -> Result<Option<User>, RequestError> {
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
        let result = system::action::get_user_from_session_token(value, conn)?;
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

    let split = header.split(" ").collect::<Vec<&str>>();
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
        return is_authed(value, repo, conn);
    }
    Ok(false)
}
pub fn can_read_basic_auth(
    header_map: &HeaderMap,
    repo: &Repository,
    conn: &MysqlConnection,
) -> Result<bool, InternalError> {
     match repo.security.visibility {
        Visibility::Public => {
            return Ok(true)
        }
        Visibility::Private => {
            let option = header_map.get("Authorization");
            if option.is_none() {
                return Ok(false);
            }
            let x = option.unwrap().to_str();
            if x.is_err() {}
            let header = x.unwrap().to_string();

            let split = header.split(" ").collect::<Vec<&str>>();
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
                return is_authed(value, repo, conn);
            }
            Ok(false)
        }
        Visibility::Hidden => {
            return Ok(true)
        }
    }

}

pub fn is_authed(
    user: String,
    repo: &Repository,
    conn: &MysqlConnection,
) -> Result<bool, InternalError> {
    let result = base64::decode(user)?;
    let string = String::from_utf8(result)?;
    let split = string.split(":").collect::<Vec<&str>>();

    if !split.len().eq(&2) {
        return Ok(false);
    }
    let result1 = get_user_by_username(split.get(0).unwrap().to_string(), &conn)?;
    if result1.is_none() {
        return Ok(false);
    }
    let argon2 = Argon2::default();
    let user = result1.unwrap();
    let parsed_hash = PasswordHash::new(user.password.as_str())?;
    if argon2
        .verify_password(split.get(1).unwrap().clone().as_bytes(), &parsed_hash)
        .is_err()
    {
        return Ok(false);
    }
    if !user.permissions.admin {
        if repo.security.open_to_all_deployers {
            if user.permissions.deployer {
                return Ok(true);
            }
        } else {
            return Ok(repo.security.deployers.contains(&user.id));
        }
    }
    return Ok(true);
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct NewUser {
    pub name: String,
    pub username: Option<String>,
    pub email: Option<String>,
    pub password: Option<NewPassword>,
    pub permissions: UserPermissions,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ModifyUser {
    pub name: String,
    pub email: String,
    pub permissions: UserPermissions,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct NewPassword {
    pub password: String,
    pub password_two: String,
}

impl NewPassword {
    pub fn hash(&self) -> Result<String, RequestError> {
        if self.password != self.password_two {
            return Err(RequestError::from("Mismatching Password"));
        }
        let salt = SaltString::generate(&mut OsRng);

        let argon2 = Argon2::default();
        let password_hash = argon2
            .hash_password(self.password.as_bytes(), salt.as_ref())
            .unwrap()
            .to_string();
        return Ok(password_hash);
    }
}

pub fn new_user(new_user: NewUser, conn: &MysqlConnection) -> Result<User, RequestError> {
    let username = new_user
        .username
        .ok_or(MissingArgument("Username".into()))?;
    let option = system::action::get_user_by_username(username.clone(), &conn)?;
    if option.is_some() {
        return Err(RequestError::Error("Username Already Exists".into()));
    }
    let email = new_user.email.ok_or(MissingArgument("Email".into()))?;
    let option = system::action::get_user_by_email(email.clone(), &conn)?;
    if option.is_some() {
        return Err(RequestError::from("Email Already Exists"));
    }

    let user = User {
        id: 0,
        name: new_user.name.clone(),
        username: username.clone(),
        email: email.clone(),
        password: new_user
            .password
            .ok_or(MissingArgument("Missing Password".into()))?
            .hash()?,
        permissions: new_user.permissions.clone(),
        created: get_current_time(),
    };
    add_new_user(&user, &conn)?;
    return Ok(
        get_user_by_username(username, &conn)?.ok_or(RequestError::from("Unable to find new user"))?
    );
}

pub fn generate_session_token(connection: &MysqlConnection) -> Result<String, RequestError> {
    loop {
        let x: String = OsRng
            .sample_iter(&Alphanumeric)
            .take(128)
            .map(char::from)
            .collect();
        let result = get_session_token(x.clone(), &connection)?;
        if result.is_none() {
            return Ok(x);
        }
    }
}

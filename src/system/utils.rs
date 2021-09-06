use actix_web::http::HeaderMap;

use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use diesel::MysqlConnection;
use serde::{Deserialize, Serialize};

use crate::apierror::APIError;
use crate::apierror::APIError::MissingArgument;
use crate::error::internal_error::InternalError;
use crate::error::request_error::RequestError;
use crate::repository::models::Repository;
use crate::system;
use crate::system::action::{add_new_user, get_auth_token, get_user_by_username};
use crate::system::models::{User, UserPermissions};
use crate::utils::get_current_time;
use rand::distributions::Alphanumeric;
use rand::Rng;

pub fn get_user_by_header(
    header_map: &HeaderMap,
    conn: &MysqlConnection,
) -> Result<Option<User>, APIError> {
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
        return can_deploy(value, repo, conn);
    }
    Ok(false)
}

pub fn can_deploy(
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
        if let Some(settings) = &repo.settings.security_rules {
            if settings.open_to_all_deployers {
                if user.permissions.deployer {
                    return Ok(true);
                }
            } else {
                return Ok(settings.deployers.contains(&user.id));
            }
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
    pub fn hash(&self) -> Result<String, APIError> {
        if self.password != self.password_two {
            return Err(APIError::from("Mismatching Password"));
        }
        let salt = SaltString::generate(&mut OsRng);

        let argon2 = Argon2::default();
        let password_hash = argon2
            .hash_password_simple(self.password.as_bytes(), salt.as_ref())
            .unwrap()
            .to_string();
        return Ok(password_hash);
    }
}

pub fn new_user(new_user: NewUser, conn: &MysqlConnection) -> Result<User, APIError> {
    let username = new_user
        .username
        .ok_or(MissingArgument("Username".into()))?;
    let option = system::action::get_user_by_username(username.clone(), &conn)?;
    if option.is_some() {
        return Err(APIError::Error("Username Already Exists".into()));
    }
    let email = new_user.email.ok_or(MissingArgument("Email".into()))?;
    let option = system::action::get_user_by_email(email.clone(), &conn)?;
    if option.is_some() {
        return Err(APIError::from("Email Already Exists"));
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
        get_user_by_username(username, &conn)?.ok_or(APIError::from("Unable to find new user"))?
    );
}

pub fn generate_auth_token(connection: &MysqlConnection) -> Result<String, RequestError> {
    loop {
        let x: String = OsRng
            .sample_iter(&Alphanumeric)
            .take(128)
            .map(char::from)
            .collect();
        let result = get_auth_token(x.clone(), &connection)?;
        if result.is_none() {
            return Ok(x);
        }
    }
}

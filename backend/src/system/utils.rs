use actix_web::http::header::HeaderMap;
use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};

use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};

use crate::error::internal_error::InternalError;
use crate::repository::models::Repository;
use crate::repository::settings::security::Visibility;
use crate::system::permissions::{can_deploy, can_read, UserPermissions};
use crate::system::user;
use crate::system::user::Entity as UserEntity;
use crate::system::user::Model as UserModel;

pub async fn verify_login(
    username: String,
    password: String,
    database: &DatabaseConnection,
) -> Result<Option<UserModel>, InternalError> {
    let user_found: Option<UserModel> = UserEntity::find()
        .filter(user::Column::Username.eq(username))
        .one(database)
        .await?;
    if user_found.is_none() {
        return Ok(None);
    }
    let argon2 = Argon2::default();
    let user = user_found.unwrap();
    let parsed_hash = PasswordHash::new(user.password.as_str())?;
    if argon2
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_err()
    {
        return Ok(None);
    }
    return Ok(Some(user));
}

pub async fn get_user_by_header(
    header_map: &HeaderMap,
    conn: &DatabaseConnection,
) -> Result<Option<UserModel>, InternalError> {
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
        todo!("Bearer Needs to be completed")
        //let result = system::action::get_user_from_session_token(&value, conn)?;
        //return Ok(result);
    } else if key.eq("Basic") {
        let result = base64::decode(value)?;
        let string = String::from_utf8(result)?;
        let split = string.split(':').collect::<Vec<&str>>();

        if !split.len().eq(&2) {
            return Ok(None);
        }
        let username = split.get(0).unwrap().to_string();
        let password = split.get(1).unwrap().to_string();
        return verify_login(username, password, conn).await;
    }
    Ok(None)
}

pub async fn can_deploy_basic_auth(
    header_map: &HeaderMap,
    repo: &Repository,
    conn: &DatabaseConnection,
) -> Result<(bool, Option<UserModel>), InternalError> {
    let option = get_user_by_header(header_map, conn).await?;
    if option.is_none() {
        return Ok((false, None));
    }
    let user = option.unwrap();
    let value = UserPermissions::try_from(user.permissions.clone())?;
    Ok((can_deploy(&value, repo)?, Some(user)))
}

pub async fn can_read_basic_auth(
    header_map: &HeaderMap,
    repo: &Repository,
    conn: &DatabaseConnection,
) -> Result<(bool, Option<UserModel>), InternalError> {
    match repo.security.visibility {
        Visibility::Public => Ok((true, None)),
        Visibility::Private => {
            let option = get_user_by_header(header_map, conn).await?;
            if option.is_none() {
                return Ok((false, None));
            }
            let user = option.unwrap();
            let value = UserPermissions::try_from(user.permissions.clone())?;

            Ok((can_read(&value, repo)?, Some(user)))
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

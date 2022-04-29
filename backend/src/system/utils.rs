use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};

use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};

use crate::error::internal_error::InternalError;
use crate::system::user;
use crate::system::user::{UserEntity, UserModel};

pub async fn verify_login(
    username: String,
    password: String,
    database: &DatabaseConnection,
) -> Result<Option<UserModel>, InternalError> {
    let user_found: Option<UserModel> = user::get_by_username(&username,database).await?;
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
    Ok(Some(user))
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

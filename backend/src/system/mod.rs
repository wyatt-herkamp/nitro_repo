use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHasher};
use rand_core::OsRng;

use crate::error::internal_error::InternalError;

pub mod permissions;
pub mod user;
pub mod web;

pub fn hash(password: String) -> Result<String, InternalError> {
    let salt = SaltString::generate(&mut OsRng);

    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(password.as_bytes(), salt.as_ref())?
        .to_string();
    Ok(password_hash)
}

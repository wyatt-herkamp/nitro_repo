use argon2::{Argon2, PasswordHash, PasswordVerifier};
use diesel::MysqlConnection;

use crate::error::internal_error::InternalError;
use crate::repository::npm::models::LoginRequest;
use crate::system::action::get_user_by_username;

pub fn is_valid(
    username: &String,
    request: &LoginRequest,
    conn: &MysqlConnection,
) -> Result<bool, InternalError> {
    let result1 = get_user_by_username(username, conn)?;
    if result1.is_none() {
        return Ok(false);
    }
    let argon2 = Argon2::default();
    let user = result1.unwrap();
    let parsed_hash = PasswordHash::new(user.password.as_str())?;
    if argon2
        .verify_password(request.password.as_bytes(), &parsed_hash)
        .is_err()
    {
        return Ok(false);
    }
    return Ok(true);
}

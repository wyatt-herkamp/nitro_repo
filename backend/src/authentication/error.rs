use sea_orm::DbErr;

use crate::error::internal_error::InternalError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AuthenticationError {
    #[error("Internal Auth Error {0}")]
    InternalError(String),
}

impl From<&str> for AuthenticationError {
    fn from(err: &str) -> AuthenticationError {
        AuthenticationError::InternalError(err.to_string())
    }
}

impl From<String> for AuthenticationError {
    fn from(err: String) -> AuthenticationError {
        AuthenticationError::InternalError(err)
    }
}

impl From<DbErr> for AuthenticationError {
    fn from(err: DbErr) -> AuthenticationError {
        AuthenticationError::InternalError(err.to_string())
    }
}
impl From<argon2::password_hash::Error> for AuthenticationError {
    fn from(err: argon2::password_hash::Error) -> AuthenticationError {
        AuthenticationError::InternalError(err.to_string())
    }
}

impl From<AuthenticationError> for InternalError {
    fn from(error: AuthenticationError) -> Self {
        InternalError::Error(error.to_string())
    }
}

use std::str::ParseBoolError;
use std::string::FromUtf8Error;

use base64::DecodeError;
use this_actix_error::ActixError;
use thiserror::Error;

use crate::app::authentication::session::SessionError;

/// Errors that happen internally to the system.
/// Not as a direct result of a Request
#[derive(Error, Debug, ActixError)]
pub enum InternalError {
    #[error("JSON error {0}")]
    JSONError(#[from] serde_json::Error),
    #[error("IO error {0}")]
    IOError(#[from] std::io::Error),
    #[error("DB error {0}")]
    DBError(#[from] sqlx::Error),
    #[error("Boolean Parse Error")]
    BooleanParseError(#[from] ParseBoolError),
    #[error("Decode Error")]
    DecodeError(#[from] DecodeError),
    #[error("UTF Decode Error")]
    UTF8Error(#[from] FromUtf8Error),
    #[error("Internal Error {0}")]
    Error(String),
    #[error("Missing Config Value {0}")]
    ConfigError(String),
    #[error("Invalid Repository Type {0}")]
    InvalidRepositoryType(String),
    #[error("Session Error {0}")]
    SessionError(#[from] SessionError),
}
impl From<argon2::password_hash::Error> for InternalError {
    fn from(err: argon2::password_hash::Error) -> InternalError {
        InternalError::Error(err.to_string())
    }
}

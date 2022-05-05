use std::error::Error;
use std::str::ParseBoolError;
use std::string::FromUtf8Error;
use std::time::SystemTimeError;

use crate::storage::error::StorageError;

use actix_web::http::StatusCode;
use actix_web::{HttpResponse, HttpResponseBuilder, ResponseError};
use base64::DecodeError;
use thiserror::Error;
/// Errors that happen internally to the system.
/// Not as a direct result of a Request
#[derive(Error, Debug)]
pub enum InternalError {
    #[error("JSON error {0}")]
    JSONError(serde_json::Error),
    #[error("IO error {0}")]
    IOError(std::io::Error),
    #[error("DB error {0}")]
    DBError(sea_orm::error::DbErr),
    #[error("Boolean Parse Error")]
    BooleanParseError(ParseBoolError),
    #[error("Decode Error")]
    DecodeError(DecodeError),
    #[error("UTF Decode Error")]
    UTF8Error(FromUtf8Error),
    #[error("SMTP Error")]
    SMTPTransportError(lettre::transport::smtp::Error),
    #[error("Internal Error {0}")]
    Error(String),
    #[error("Missing Config Value {0}")]
    ConfigError(String),
    #[error("Storage Error: {0}")]
    StorageError(StorageError),
    #[error("Invalid Repository Type {0}")]
    InvalidRepositoryType(String),
}
impl From<StorageError> for InternalError {
    fn from(storage_error: StorageError) -> Self {
        InternalError::StorageError(storage_error)
    }
}

impl From<DecodeError> for InternalError {
    fn from(err: DecodeError) -> InternalError {
        InternalError::DecodeError(err)
    }
}

impl From<chrono::ParseError> for InternalError {
    fn from(err: chrono::ParseError) -> InternalError {
        InternalError::Error(err.to_string())
    }
}

impl From<std::io::Error> for InternalError {
    fn from(err: std::io::Error) -> InternalError {
        InternalError::IOError(err)
    }
}

impl From<serde_json::Error> for InternalError {
    fn from(err: serde_json::Error) -> InternalError {
        InternalError::JSONError(err)
    }
}

impl From<FromUtf8Error> for InternalError {
    fn from(err: FromUtf8Error) -> InternalError {
        InternalError::UTF8Error(err)
    }
}

impl From<Box<dyn Error>> for InternalError {
    fn from(err: Box<dyn Error>) -> InternalError {
        InternalError::Error(err.to_string())
    }
}

impl From<sea_orm::DbErr> for InternalError {
    fn from(err: sea_orm::DbErr) -> InternalError {
        InternalError::DBError(err)
    }
}

impl From<argon2::password_hash::Error> for InternalError {
    fn from(err: argon2::password_hash::Error) -> InternalError {
        InternalError::Error(err.to_string())
    }
}

impl From<handlebars::RenderError> for InternalError {
    fn from(err: handlebars::RenderError) -> InternalError {
        InternalError::Error(err.to_string())
    }
}

impl From<actix_web::Error> for InternalError {
    fn from(err: actix_web::Error) -> InternalError {
        InternalError::ActixWebError(err)
    }
}

impl From<SystemTimeError> for InternalError {
    fn from(err: SystemTimeError) -> InternalError {
        InternalError::Error(err.to_string())
    }
}

impl From<lettre::transport::smtp::Error> for InternalError {
    fn from(err: lettre::transport::smtp::Error) -> InternalError {
        InternalError::SMTPTransportError(err)
    }
}

impl From<ParseBoolError> for InternalError {
    fn from(err: ParseBoolError) -> InternalError {
        InternalError::BooleanParseError(err)
    }
}

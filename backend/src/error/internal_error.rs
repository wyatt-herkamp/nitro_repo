use std::error::Error;
use std::str::ParseBoolError;
use std::string::FromUtf8Error;
use std::time::SystemTimeError;

use actix_web::http::StatusCode;
use actix_web::HttpResponse;
use base64::DecodeError;
use handlebars::RenderError;
use this_actix_error::ActixError;
use thiserror::Error;
use crate::system::permissions::PermissionError;

#[derive(Error, Debug, ActixError)]
pub enum InternalError {
    #[error("JSON error {0}")]
    JSONError(#[from]serde_json::Error),
    #[error("IO error {0}")]
    IOError(#[from]std::io::Error),
    #[error("DB error {0}")]
    DBError(#[from]diesel::result::Error),
    #[error("Actix Error")]
    ActixWebError(#[from]actix_web::Error),
    #[error("R2d2 Parse Error")]
    R2D2Error(#[from]r2d2::Error),
    #[status_code(400)]
    #[error("Decode Error")]
    DecodeError(#[from]DecodeError),
    #[status_code(400)]
    #[message("Not UTF-8 Frmat")]
    #[error("UTF Decode Error")]
    UTF8Error(#[from]FromUtf8Error),
    #[error("SMTP Error")]
    SMTPTransportError(#[from]lettre::transport::smtp::Error),
    #[error("Missing Argument {0}")]
    MissingArgument(String),
    #[error("Not Found")]
    NotFound,
    #[error("Internal Error {0}")]
    Error(String),
    #[error("Missing Config Value {0}")]
    ConfigError(String),
    #[error("Invalid Repository Type {0}")]
    InvalidRepositoryType(String),
    #[status_code(403)]
    #[error("Permission Error: {0}")]
    PermissionError(#[from]crate::system::permissions::PermissionError),
    #[error("Internal Error: {0}")]
    RenderError(#[from] RenderError),
    #[status_code(400)]
    #[message("Unable to Parse the Date Format")]
    #[error("Chrono Parse Error")]
    ChronoParseError(#[from] chrono::ParseError),
    #[error("Unable to convert date")]
    SystemTimeError(#[from] SystemTimeError),
}

pub type NResult<T> = Result<T, InternalError>;

impl InternalError {
    pub fn json_error(&self) -> HttpResponse {
        let result = HttpResponse::Ok()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .content_type("text/plain")
            .body(self.to_string());
        result
    }
}

impl From<argon2::password_hash::Error> for InternalError {
    fn from(err: argon2::password_hash::Error) -> Self {
        InternalError::Error(err.to_string())
    }
}

impl From<&str> for InternalError {
    fn from(error: &str) -> Self {
        InternalError::Error(error.to_string())
    }
}

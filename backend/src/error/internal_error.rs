use std::error::Error;
use std::str::ParseBoolError;
use std::string::FromUtf8Error;
use std::time::SystemTimeError;

use actix_web::http::StatusCode;
use actix_web::HttpResponse;
use base64::DecodeError;
use thiserror::Error;
use crate::system::permissions::PermissionError;

#[derive(Error, Debug)]
pub enum InternalError {
    #[error("JSON error {0}")]
    JSONError(serde_json::Error),
    #[error("IO error {0}")]
    IOError(std::io::Error),
    #[error("DB error {0}")]
    DBError(diesel::result::Error),
    #[error("Actix Error")]
    ActixWebError(actix_web::Error),
    #[error("R2d2 Parse Error")]
    R2D2Error(r2d2::Error),
    #[error("Boolean Parse Error")]
    BooleanParseError(ParseBoolError),
    #[error("Decode Error")]
    DecodeError(DecodeError),
    #[error("UTF Decode Error")]
    UTF8Error(FromUtf8Error),
    #[error("SMTP Error")]
    SMTPTransportError(lettre::transport::smtp::Error),
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
    #[error("Permission Error: {0}")]
    PermissionError(crate::system::permissions::PermissionError),
    #[error("THE INTERNAL WEBSITE HAS BROKEN DOWN. PLEASE REPORT to https://github.com/wherkamp/nitro_repo and restart application")]
    DeadSite,
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

impl actix_web::error::ResponseError for InternalError {
    fn error_response(&self) -> HttpResponse {
        self.json_error()
    }
}

//from<Error>
impl From<PermissionError> for InternalError {
    fn from(err: PermissionError) -> InternalError {
        InternalError::PermissionError(err)
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

impl From<diesel::result::Error> for InternalError {
    fn from(err: diesel::result::Error) -> InternalError {
        InternalError::DBError(err)
    }
}

impl From<r2d2::Error> for InternalError {
    fn from(err: r2d2::Error) -> InternalError {
        InternalError::R2D2Error(err)
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

impl From<serde_json::Error> for InternalError {
    fn from(err: serde_json::Error) -> InternalError {
        InternalError::JSONError(err)
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

impl From<std::io::Error> for InternalError {
    fn from(err: std::io::Error) -> InternalError {
        InternalError::IOError(err)
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

impl From<&str> for InternalError {
    fn from(error: &str) -> Self {
        InternalError::Error(error.to_string())
    }
}

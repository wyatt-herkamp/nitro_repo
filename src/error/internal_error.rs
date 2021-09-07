

use std::str::{FromStr, ParseBoolError};




use actix_web::{HttpResponse};
use derive_more::{Display, Error};



use crate::error::GenericError;
use crate::repository::repo_error::RepositoryError;
use base64::DecodeError;
use std::string::FromUtf8Error;
use actix_web::http::StatusCode;

#[derive(Debug, Display, Error)]
pub enum InternalError {
    JSONError(serde_json::Error),
    DBError(diesel::result::Error),
    ActixWebError(actix_web::Error),
    R2D2Error(r2d2::Error),
    BooleanParseError(ParseBoolError),
    DecodeError(DecodeError),
    UTF8Error(FromUtf8Error),
    SMTPTransportError(lettre::transport::smtp::Error),
    MissingArgument(GenericError),
    Error(GenericError),
    UnInstalled,
    RepoError(RepositoryError),
}

impl InternalError {
    pub fn json_error(&self) -> HttpResponse {
        let result = HttpResponse::Ok()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .content_type("application/json")
            .body("");
        return result;
    }
}

//from<Error>
impl From<DecodeError> for InternalError {
    fn from(err: DecodeError) -> InternalError {
        return InternalError::DecodeError(err);
    }
}

impl From<FromUtf8Error> for InternalError {
    fn from(err: FromUtf8Error) -> InternalError {
        return InternalError::UTF8Error(err);
    }
}

impl From<diesel::result::Error> for InternalError {
    fn from(err: diesel::result::Error) -> InternalError {
        return InternalError::DBError(err);
    }
}

impl From<r2d2::Error> for InternalError {
    fn from(err: r2d2::Error) -> InternalError {
        InternalError::R2D2Error(err)
    }
}
impl From<argon2::password_hash::Error> for InternalError {
    fn from(err: argon2::password_hash::Error) -> InternalError {
        InternalError::Error(GenericError::from(err.to_string()))
    }
}

impl FromStr for InternalError {
    type Err = InternalError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let error = GenericError {
            error: s.to_string(),
        };
        Ok(InternalError::Error(error))
    }
}

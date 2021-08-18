use std::error::Error;
use std::fmt::{Display, Formatter};
use std::str::{FromStr, ParseBoolError};

use actix_web::body::Body;
use actix_web::dev::HttpResponseBuilder;
use actix_web::http::header::ToStrError;
use actix_web::HttpResponse;
use derive_more::{Display, Error};
use hyper::StatusCode;

use crate::api_response::{APIErrorResponse, APIResponse};
use crate::apierror::APIError;
use crate::repository::repo_error::RepositoryError;

#[derive(Debug, Display, Error)]
pub enum InternalError {
    JSONError(serde_json::Error),
    DBError(diesel::result::Error),
    ActixWebError(actix_web::Error),
    R2D2Error(r2d2::Error),
    BooleanParseError(ParseBoolError),
    HyperError(hyper::Error),
    TeraError(tera::Error),
    SMTPTransportError(lettre::transport::smtp::Error),
    MissingArgument(GenericError),
    Error(GenericError),
    Internal(APIError),
    UnInstalled,
    RepoError(RepositoryError),
}

#[derive(Debug)]
pub struct GenericError {
    pub error: String,
}

impl Display for GenericError {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "Error! {}. ({:?})", self.error, self.error)
    }
}

impl Error for GenericError {
    fn description(&self) -> &str {
        self.error.as_str()
    }
}

impl From<String> for GenericError {
    fn from(value: String) -> Self {
        GenericError { error: value }
    }
}


impl From<&str> for GenericError {
    fn from(value: &str) -> Self {
        GenericError {
            error: value.to_string(),
        }
    }
}

impl FromStr for GenericError {
    type Err = InternalError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(GenericError {
            error: s.to_string(),
        })
    }
}

impl actix_web::error::ResponseError for InternalError {
    fn error_response(&self) -> HttpResponse {
        log::error!("Site Error: {}", self.to_string());
        HttpResponse::Ok().status(StatusCode::INTERNAL_SERVER_ERROR).content_type("text/html").
            body(crate::utils::Resources::file_get_string("pages/error/500.html"))
    }
}

impl From<diesel::result::Error> for InternalError {
    fn from(err: diesel::result::Error) -> InternalError {
        InternalError::DBError(err)
    }
}

impl From<serde_json::Error> for InternalError {
    fn from(err: serde_json::Error) -> InternalError {
        InternalError::JSONError(err)
    }
}

impl From<tera::Error> for InternalError {
    fn from(err: tera::Error) -> InternalError {
        InternalError::TeraError(err)
    }
}

impl From<actix_web::Error> for InternalError {
    fn from(err: actix_web::Error) -> InternalError {
        InternalError::ActixWebError(err)
    }
}

impl From<r2d2::Error> for InternalError {
    fn from(err: r2d2::Error) -> InternalError {
        InternalError::R2D2Error(err)
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

impl From<hyper::Error> for InternalError {
    fn from(err: hyper::Error) -> InternalError {
        InternalError::HyperError(err)
    }
}

impl From<RepositoryError> for InternalError {
    fn from(value: RepositoryError) -> Self {
        return InternalError::RepoError(value);
    }
}

impl From<actix_web::client::HttpError> for InternalError {
    fn from(err: actix_web::client::HttpError) -> InternalError {
        InternalError::Error(GenericError::from(err.to_string()))
    }
}

impl From<std::io::Error> for InternalError {
    fn from(err: std::io::Error) -> InternalError {
        InternalError::Error(GenericError::from(err.to_string()))
    }
}

impl From<ToStrError> for InternalError {
    fn from(err: ToStrError) -> InternalError {
        InternalError::Error(GenericError::from(err.to_string()))
    }
}

impl From<String> for InternalError {
    fn from(value: String) -> Self {
        let error = GenericError { error: value };
        InternalError::Error(error)
    }
}

impl From<APIError> for InternalError {
    fn from(value: APIError) -> Self {
        InternalError::Internal(value)
    }
}

impl From<&str> for InternalError {
    fn from(value: &str) -> Self {
        let error = GenericError {
            error: value.to_string(),
        };
        InternalError::Error(error)
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

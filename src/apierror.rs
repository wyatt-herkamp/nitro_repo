use crate::api_response::{APIErrorResponse, APIResponse};

use actix_web::http::header::ToStrError;

use crate::repository::repo_error::RepositoryError;
use actix_web::HttpResponse;
use derive_more::{Display, Error};
use hyper::StatusCode;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::str::{FromStr, ParseBoolError};

#[derive(Debug, Display, Error)]
pub enum APIError {
    JSONError(serde_json::Error),
    DBError(diesel::result::Error),
    ActixWebError(actix_web::Error),
    R2D2Error(r2d2::Error),
    BooleanParseError(ParseBoolError),
    HyperError(hyper::Error),
    SMTPTransportError(lettre::transport::smtp::Error),
    Error(GenericError),
    NotAuthorized,
    InvalidLogin,
    NotFound,
    BadRequest,
    MissingArgument(GenericError),
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
    type Err = APIError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(GenericError {
            error: s.to_string(),
        })
    }
}

impl actix_web::error::ResponseError for APIError {
    fn error_response(&self) -> HttpResponse {
        return match self {
            APIError::NotAuthorized => {
                let error = APIErrorResponse {
                    user_friendly_message: Some(self.to_string()),
                    error_code: Some("NOT_AUTHORIZED".to_string()),
                };
                APIResponse {
                    success: false,
                    data: Some(error),
                    status_code: Some(StatusCode::UNAUTHORIZED.as_u16()),
                }
                .error(StatusCode::UNAUTHORIZED)
            }
            APIError::InvalidLogin => {
                let error = APIErrorResponse {
                    user_friendly_message: Some("Invalid Login".to_string()),
                    error_code: Some("NOT_AUTHORIZED".to_string()),
                };
                APIResponse {
                    success: false,
                    data: Some(error),
                    status_code: Some(StatusCode::UNAUTHORIZED.as_u16()),
                }
                .error(StatusCode::UNAUTHORIZED)
            }
            APIError::BadRequest => {
                let error = APIErrorResponse {
                    user_friendly_message: Some(self.to_string()),
                    error_code: Some("BAD_REQUEST".to_string()),
                };
                APIResponse {
                    success: false,
                    data: Some(error),
                    status_code: Some(StatusCode::BAD_REQUEST.as_u16()),
                }
                .error(StatusCode::BAD_REQUEST)
            }
            APIError::MissingArgument(s) => {
                let error = APIErrorResponse {
                    user_friendly_message: Some(format!("Missing Argument {}", s)),
                    error_code: Some("BAD_REQUEST".to_string()),
                };
                APIResponse {
                    success: false,
                    data: Some(error),
                    status_code: Some(StatusCode::BAD_REQUEST.as_u16()),
                }
                .error(StatusCode::BAD_REQUEST)
            }
            APIError::NotFound => {
                let error = APIErrorResponse {
                    user_friendly_message: Some(self.to_string()),
                    error_code: Some("NOT_FOUND".to_string()),
                };
                APIResponse {
                    success: false,
                    data: Some(error),
                    status_code: Some(StatusCode::NOT_FOUND.as_u16()),
                }
                .error(StatusCode::NOT_FOUND)
            }
            APIError::UnInstalled => {
                let error = APIErrorResponse {
                    user_friendly_message: Some(self.to_string()),
                    error_code: Some("UNINSTALLED".to_string()),
                };
                APIResponse {
                    success: false,
                    data: Some(error),
                    status_code: Some(StatusCode::OK.as_u16()),
                }
                .error(StatusCode::OK)
            }
            _ => {
                let error = APIErrorResponse {
                    user_friendly_message: Some(self.to_string()),
                    error_code: None,
                };
                APIResponse {
                    success: false,
                    data: Some(error),
                    status_code: Some(StatusCode::INTERNAL_SERVER_ERROR.as_u16()),
                }
                .error(StatusCode::INTERNAL_SERVER_ERROR)
            }
        };
    }
}

impl From<diesel::result::Error> for APIError {
    fn from(err: diesel::result::Error) -> APIError {
        APIError::DBError(err)
    }
}

impl From<serde_json::Error> for APIError {
    fn from(err: serde_json::Error) -> APIError {
        APIError::JSONError(err)
    }
}

impl From<actix_web::Error> for APIError {
    fn from(err: actix_web::Error) -> APIError {
        APIError::ActixWebError(err)
    }
}

impl From<r2d2::Error> for APIError {
    fn from(err: r2d2::Error) -> APIError {
        APIError::R2D2Error(err)
    }
}
impl From<lettre::transport::smtp::Error> for APIError {
    fn from(err: lettre::transport::smtp::Error) -> APIError {
        APIError::SMTPTransportError(err)
    }
}

impl From<ParseBoolError> for APIError {
    fn from(err: ParseBoolError) -> APIError {
        APIError::BooleanParseError(err)
    }
}

impl From<hyper::Error> for APIError {
    fn from(err: hyper::Error) -> APIError {
        APIError::HyperError(err)
    }
}
impl From<RepositoryError> for APIError {
    fn from(value: RepositoryError) -> Self {
        return APIError::RepoError(value);
    }
}
impl From<actix_web::client::HttpError> for APIError {
    fn from(err: actix_web::client::HttpError) -> APIError {
        APIError::Error(GenericError::from(err.to_string()))
    }
}

impl From<std::io::Error> for APIError {
    fn from(err: std::io::Error) -> APIError {
        APIError::Error(GenericError::from(err.to_string()))
    }
}

impl From<ToStrError> for APIError {
    fn from(err: ToStrError) -> APIError {
        APIError::Error(GenericError::from(err.to_string()))
    }
}

impl From<String> for APIError {
    fn from(value: String) -> Self {
        let error = GenericError { error: value };
        APIError::Error(error)
    }
}

impl From<&str> for APIError {
    fn from(value: &str) -> Self {
        let error = GenericError {
            error: value.to_string(),
        };
        APIError::Error(error)
    }
}

impl FromStr for APIError {
    type Err = APIError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let error = GenericError {
            error: s.to_string(),
        };
        Ok(APIError::Error(error))
    }
}

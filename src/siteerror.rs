use crate::api_response::{APIError, APIResponse};

use actix_web::http::header::ToStrError;

use actix_web::HttpResponse;
use derive_more::{Display, Error};
use hyper::StatusCode;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::str::{FromStr, ParseBoolError};
use crate::repository::repo_error::RepositoryError;

#[derive(Debug, Display, Error)]
pub enum SiteError {
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
    RepoError(RepositoryError)
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
    type Err = SiteError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(GenericError {
            error: s.to_string(),
        })
    }
}

impl actix_web::error::ResponseError for SiteError {
    fn error_response(&self) -> HttpResponse {
        return match self {
            SiteError::NotAuthorized => {
                let error = APIError {
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
            SiteError::InvalidLogin => {
                let error = APIError {
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
            SiteError::BadRequest => {
                let error = APIError {
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
            SiteError::MissingArgument(s) => {
                let error = APIError {
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
            SiteError::NotFound => {
                let error = APIError {
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
            SiteError::UnInstalled => {
                let error = APIError {
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
                let error = APIError {
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

impl From<diesel::result::Error> for SiteError {
    fn from(err: diesel::result::Error) -> SiteError {
        SiteError::DBError(err)
    }
}

impl From<serde_json::Error> for SiteError {
    fn from(err: serde_json::Error) -> SiteError {
        SiteError::JSONError(err)
    }
}

impl From<actix_web::Error> for SiteError {
    fn from(err: actix_web::Error) -> SiteError {
        SiteError::ActixWebError(err)
    }
}

impl From<r2d2::Error> for SiteError {
    fn from(err: r2d2::Error) -> SiteError {
        SiteError::R2D2Error(err)
    }
}
impl From<lettre::transport::smtp::Error> for SiteError {
    fn from(err: lettre::transport::smtp::Error) -> SiteError {
        SiteError::SMTPTransportError(err)
    }
}

impl From<ParseBoolError> for SiteError {
    fn from(err: ParseBoolError) -> SiteError {
        SiteError::BooleanParseError(err)
    }
}

impl From<hyper::Error> for SiteError {
    fn from(err: hyper::Error) -> SiteError {
        SiteError::HyperError(err)
    }
}
impl From<RepositoryError> for SiteError {
    fn from(value: RepositoryError) -> Self {
        return SiteError::RepoError(value);
    }
}
impl From<actix_web::client::HttpError> for SiteError {
    fn from(err: actix_web::client::HttpError) -> SiteError {
        SiteError::Error(GenericError::from(err.to_string()))
    }
}

impl From<std::io::Error> for SiteError {
    fn from(err: std::io::Error) -> SiteError {
        SiteError::Error(GenericError::from(err.to_string()))
    }
}

impl From<ToStrError> for SiteError {
    fn from(err: ToStrError) -> SiteError {
        SiteError::Error(GenericError::from(err.to_string()))
    }
}

impl From<String> for SiteError {
    fn from(value: String) -> Self {
        let error = GenericError { error: value };
        SiteError::Error(error)
    }
}

impl From<&str> for SiteError {
    fn from(value: &str) -> Self {
        let error = GenericError {
            error: value.to_string(),
        };
        SiteError::Error(error)
    }
}

impl FromStr for SiteError {
    type Err = SiteError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let error = GenericError {
            error: s.to_string(),
        };
        Ok(SiteError::Error(error))
    }
}

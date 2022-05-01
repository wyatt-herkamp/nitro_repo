use std::error::Error;
use std::fmt::{Display, Formatter};
use actix_web::http::StatusCode;
use actix_web::{HttpResponse, ResponseError};
use actix_web::body::BoxBody;
use crate::authentication::UnAuthorized;
use crate::system::permissions::options::MissingPermission;
use crate::system::permissions::PermissionError;
use thiserror::Error;
use crate::repository::maven::error::MavenError;
use crate::repository::npm::error::NPMError;
use crate::storage::models::StorageError;

#[derive(Debug)]
pub enum RepositoryError {
    InternalError(String),
    RequestError(String, StatusCode),
    MavenError(MavenError),
    NPMError(NPMError),
}

impl Display for RepositoryError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            RepositoryError::InternalError(internal) => {
                write!(f, "Internal Repository Error {}", internal)
            }
            RepositoryError::RequestError(request, _) => {
                write!(f, "Bad Repository  Error {}", request)
            }

            RepositoryError::MavenError(error) => write!(f, "{}", error),
            RepositoryError::NPMError(error) => write!(f, "{}", error)
        }
    }
}

impl Error for RepositoryError {}

impl From<std::io::Error> for RepositoryError {
    fn from(err: std::io::Error) -> RepositoryError {
        RepositoryError::InternalError(err.to_string())
    }
}

impl From<&str> for RepositoryError {
    fn from(err: &str) -> RepositoryError {
        RepositoryError::InternalError(err.to_string())
    }
}

impl From<String> for RepositoryError {
    fn from(err: String) -> RepositoryError {
        RepositoryError::InternalError(err)
    }
}

impl From<serde_json::Error> for RepositoryError {
    fn from(err: serde_json::Error) -> RepositoryError {
        RepositoryError::InternalError(err.to_string())
    }
}

impl From<StorageError> for RepositoryError {
    fn from(err: StorageError) -> RepositoryError {
        RepositoryError::InternalError(err.to_string())
    }
}
impl From<UnAuthorized> for RepositoryError {
    fn from(err: UnAuthorized) -> RepositoryError {
        RepositoryError::RequestError("Not Authorized".to_string(), StatusCode::NOT_FOUND)
    }
}

impl ResponseError for RepositoryError {
    fn status_code(&self) -> StatusCode {
        match self {
            RepositoryError::InternalError(_) => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
            RepositoryError::RequestError(_, status) => {
                status.clone()
            }
            _ => {
                StatusCode::BAD_REQUEST
            }
        }
    }
    fn error_response(&self) -> HttpResponse<BoxBody> {
        HttpResponse::with_body(self.status_code(), self.to_string()).map_into_boxed_body()
    }
}

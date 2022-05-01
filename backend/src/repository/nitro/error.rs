use crate::repository::error::RepositoryError;
use crate::storage::models::StorageError;
use actix_web::http::StatusCode;
use std::string;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum NitroError {
    #[error("Internal Error: {0}")]
    InternalError(String),
    #[error("Project Requested Not Found")]
    ProjectNotFound,
}

impl From<&str> for NitroError {
    fn from(err: &str) -> NitroError {
        NitroError::InternalError(err.to_string())
    }
}

impl From<String> for NitroError {
    fn from(err: String) -> NitroError {
        NitroError::InternalError(err)
    }
}

impl From<StorageError> for NitroError {
    fn from(err: StorageError) -> NitroError {
        NitroError::InternalError(err.to_string())
    }
}

impl From<string::FromUtf8Error> for NitroError {
    fn from(err: string::FromUtf8Error) -> NitroError {
        NitroError::InternalError(err.to_string())
    }
}

impl From<serde_json::Error> for NitroError {
    fn from(err: serde_json::Error) -> NitroError {
        NitroError::InternalError(err.to_string())
    }
}

impl From<NitroError> for RepositoryError {
    fn from(error: NitroError) -> RepositoryError {
        match error {
            NitroError::InternalError(internal) => RepositoryError::InternalError(internal),
            NitroError::ProjectNotFound => {
                RepositoryError::RequestError(error.to_string(), StatusCode::NOT_FOUND)
            }
        }
    }
}

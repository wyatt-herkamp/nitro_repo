use std::borrow::Cow;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};

use actix_web::body::BoxBody;
use actix_web::http::StatusCode;
use actix_web::{HttpResponse, HttpResponseBuilder, ResponseError};
use serde_json::json;
#[deprecated]
#[derive(Debug)]
pub struct APIError<'a> {
    pub message: Option<Cow<'a, str>>,
    pub status_code: StatusCode,
}
#[allow(deprecated)]
impl<'a> APIError<'a> {
    pub fn bad_request<E: Error>(error: E) -> APIError<'a> {
        APIError {
            message: Some(Cow::Owned(error.to_string())),
            status_code: StatusCode::BAD_REQUEST,
        }
    }
    pub fn internal_error<E: Error>(error: E) -> APIError<'a> {
        APIError {
            message: Some(Cow::Owned(error.to_string())),
            status_code: StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
    pub fn not_found() -> APIError<'a> {
        APIError {
            message: None,
            status_code: StatusCode::NOT_FOUND,
        }
    }
    pub fn storage_not_found() -> APIError<'a> {
        APIError {
            message: Some(Cow::Borrowed("Storage not found")),
            status_code: StatusCode::NOT_FOUND,
        }
    }
    pub fn repository_not_found() -> APIError<'a> {
        APIError {
            message: Some(Cow::Borrowed("Repository not found")),
            status_code: StatusCode::NOT_FOUND,
        }
    }
}
#[allow(deprecated)]
impl<'a> From<(&'a str, StatusCode)> for APIError<'a> {
    fn from((message, status): (&'a str, StatusCode)) -> Self {
        APIError {
            message: Some(Cow::Borrowed(message)),
            status_code: status,
        }
    }
}
#[allow(deprecated)]
impl<'a> Display for APIError<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if let Some(message) = &self.message {
            write!(f, "Status Code: {} Error: {}", self.status_code, message)
        } else {
            write!(f, "Status Code: {}", self.status_code,)
        }
    }
}
#[allow(deprecated)]
impl<'a> ResponseError for APIError<'a> {
    fn status_code(&self) -> StatusCode {
        self.status_code
    }
    fn error_response(&self) -> HttpResponse<BoxBody> {
        HttpResponseBuilder::new(self.status_code).json(json!({
            "error": self.to_string(),
        }))
    }
}

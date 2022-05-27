use actix_web::body::BoxBody;
use actix_web::http::StatusCode;

use actix_web::{HttpResponse, HttpResponseBuilder, ResponseError};
use serde_json::json;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};

pub struct APIError {
    pub message: Option<String>,
    pub status_code: StatusCode,
}
impl APIError {
    pub fn bad_request<E: Error>(error: E) -> APIError {
        APIError {
            message: Some(error.to_string()),
            status_code: StatusCode::BAD_REQUEST,
        }
    }
    pub fn internal_error<E: Error>(error: E) -> APIError {
        APIError {
            message: Some(error.to_string()),
            status_code: StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
    pub fn not_found() -> APIError {
        APIError {
            message: None,
            status_code: StatusCode::NOT_FOUND,
        }
    }
}
impl From<(&str, StatusCode)> for APIError {
    fn from((message, status): (&str, StatusCode)) -> Self {
        APIError {
            message: Some(message.to_string()),
            status_code: status,
        }
    }
}
impl Debug for APIError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if let Some(message) = &self.message {
            write!(
                f,
                "Status Code: {} Error: {}",
                self.status_code.to_string(),
                message
            )
        } else {
            write!(f, "Status Code: {}", self.status_code.to_string(),)
        }
    }
}

impl Display for APIError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if let Some(message) = &self.message {
            write!(
                f,
                "Status Code: {} Error: {}",
                self.status_code.to_string(),
                message
            )
        } else {
            write!(f, "Status Code: {}", self.status_code.to_string(),)
        }
    }
}

impl ResponseError for APIError {
    fn status_code(&self) -> StatusCode {
        self.status_code
    }
    fn error_response(&self) -> HttpResponse<BoxBody> {
        HttpResponseBuilder::new(self.status_code).json(json!({
            "error": self.to_string(),
        }))
    }
}

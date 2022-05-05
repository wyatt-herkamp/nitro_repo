use actix::fut::err;
use actix_web::body::BoxBody;
use actix_web::http::StatusCode;
use actix_web::rt::net::ActixStream;
use actix_web::{HttpRequest, HttpResponse, HttpResponseBuilder, Responder, ResponseError};
use log::error;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};

use crate::error::internal_error::InternalError;
use serde::Serialize;
use serde_json::Value;

pub type NRResponse = Result<APIResponse, APIError>;

#[derive(Debug)]
pub struct APIError(APIResponse);

impl Display for APIError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "API Error {}", self.0.status_code)
    }
}

impl ResponseError for APIError {
    fn status_code(&self) -> StatusCode {
        self.0.status_code
    }
    fn error_response(&self) -> HttpResponse<BoxBody> {
        HttpResponseBuilder::new(self.0.status_code).json(&self.0)
    }
}
impl From<APIResponse> for APIError {
    fn from(value: APIResponse) -> Self {
        Self(value)
    }
}

#[derive(Debug, Serialize)]
pub struct APIResponse {
    /// Success will be unless. The Type of data is a ResponseError
    pub success: bool,
    pub data: Option<Value>,
    #[serde(skip_serializing)]
    pub status_code: StatusCode,
}
impl Display for APIResponse {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let result = serde_json::to_string(&self).map_err(|_| std::fmt::Error)?;
        write!(f, "{}", result)
    }
}

impl APIResponse {
    pub fn ok() -> APIResponse {
        APIResponse {
            success: true,
            data: None,
            status_code: StatusCode::OK,
        }
    }
    pub fn bad_request<E: Error>(error: E) -> APIResponse {
        APIResponse {
            success: true,
            data: Some(Value::String(error.to_string())),
            status_code: StatusCode::BAD_REQUEST,
        }
    }
    pub fn not_found() -> APIResponse {
        APIResponse::from(("Not Found", StatusCode::NOT_FOUND))
    }
}
impl Into<actix_web::error::Error> for APIResponse {
    fn into(self) -> actix_web::Error {
        actix_web::error::InternalError::new(self, self.status_code.clone()).into()
    }
}
impl<T: Serialize> From<Option<T>> for APIResponse {
    fn from(data: Option<T>) -> Self {
        let status_code = if data.is_none() {
            StatusCode::NOT_FOUND
        } else {
            StatusCode::OK
        };
        (data, status_code).into()
    }
}

impl<T: Serialize> From<(Option<T>, StatusCode)> for APIResponse {
    fn from((data, status_code): (Option<T>, StatusCode)) -> Self {
        let respond_data = match data {
            None => None,
            Some(data) => serde_json::to_value(data).ok(),
        };
        APIResponse {
            success: true,
            data: respond_data,
            status_code,
        }
    }
}

impl From<(&str, StatusCode)> for APIResponse {
    fn from((friendly_code, error_code): (&str, StatusCode)) -> Self {
        APIResponse {
            success: error_code.is_success(),
            data: Some(Value::String(friendly_code.to_string())),
            status_code: error_code,
        }
    }
}

impl From<(String, StatusCode)> for APIResponse {
    fn from((friendly_code, error_code): (String, StatusCode)) -> Self {
        APIResponse {
            success: error_code.is_success(),
            data: Some(Value::String(friendly_code)),
            status_code: error_code,
        }
    }
}

impl From<InternalError> for APIResponse {
    fn from(internal_error: InternalError) -> Self {
        error!("{}", &internal_error);
        APIResponse {
            success: false,
            data: Some(Value::String(internal_error.to_string())),
            status_code: StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}
impl From<InternalError> for APIError {
    fn from(internal_error: InternalError) -> Self {
        APIResponse::from(internal_error).into()
    }
}
impl Responder for APIResponse {
    type Body = BoxBody;

    fn respond_to(self, req: &HttpRequest) -> HttpResponse<Self::Body> {
        HttpResponseBuilder::new(self.status_code)
            .json(self)
            .respond_to(req)
    }
}

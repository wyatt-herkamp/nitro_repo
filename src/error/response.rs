use actix_web::{Responder, HttpResponse};

use crate::api_response::{APIResponse, RequestErrorResponse, SiteResponse};
use actix_web::http::StatusCode;
use crate::error::internal_error::InternalError;

pub fn invalid_login() -> SiteResponse {
    (APIResponse::<RequestErrorResponse>::new(false, None).error(StatusCode::UNAUTHORIZED))
}

pub fn mismatching_passwords() -> SiteResponse {
    (APIResponse::new(false, Some(RequestErrorResponse { user_friendly_message: None, error_code: Some("MISMATCHING_PASSWORDS".to_string()) })).error(StatusCode::BAD_REQUEST))
}

pub fn not_found() -> SiteResponse {
    (APIResponse::<bool>::new(false, None).error(StatusCode::NOT_FOUND))
}

pub fn unauthorized() -> SiteResponse {
    (APIResponse::<bool>::new(false, None).error(StatusCode::UNAUTHORIZED))
}

pub fn already_exists() -> SiteResponse {
    (APIResponse::new(false, Some(RequestErrorResponse { user_friendly_message: None, error_code: Some("ALREADY_EXISTS".to_string()) })).error(StatusCode::BAD_REQUEST))
}

pub fn uninstalled() -> SiteResponse {
    (APIResponse::new(false, Some(RequestErrorResponse { user_friendly_message: None, error_code: Some("UNINSTALLED".to_string()) })).error(StatusCode::BAD_GATEWAY))
}

pub fn i_am_a_teapot(value: String) -> SiteResponse {
    (APIResponse::new(false, Some(RequestErrorResponse { user_friendly_message: Some(value), error_code: None })).error(StatusCode::IM_A_TEAPOT))
}

pub fn bad_request(value: String) -> SiteResponse {
    (APIResponse::new(false, Some(RequestErrorResponse { user_friendly_message: Some(value), error_code: None })).error(StatusCode::BAD_REQUEST))
}

pub fn missing_arguments(value: String) -> SiteResponse {
    (APIResponse::new(false, Some(RequestErrorResponse { user_friendly_message: Some(value), error_code: Some("MISSING_ARGUMENT".to_string()) })).error(StatusCode::BAD_REQUEST))
}

pub fn error(value: String, status: Option<StatusCode>) -> SiteResponse {
    (APIResponse::new(false, Some(RequestErrorResponse { user_friendly_message: Some(value), error_code: None })).error(status.unwrap_or(StatusCode::BAD_REQUEST)))
}
use std::fmt::{Display, Formatter};
use actix_web::http::StatusCode;
use actix_web::{HttpRequest, HttpResponse, Responder};

use crate::error::request_error::RequestError;
use serde::{Deserialize, Serialize};
use crate::error::internal_error::InternalError;

pub type SiteResponse = Result<HttpResponse, InternalError>;

#[derive(Debug, Serialize, Deserialize)]
pub struct APIResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub status_code: Option<u16>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RequestErrorResponse {
    //User friendly messages will be provided for some cases
    pub user_friendly_message: Option<String>,
    //Look into that specific API for what this will be set to. This is something that specific api will control
    pub error_code: Option<String>,
}impl RequestErrorResponse{
    pub fn new<S: Into<String>>(friendly: S, error: S)->RequestErrorResponse{
        return RequestErrorResponse{ user_friendly_message: Some(friendly.into()), error_code: Some(error.into()) }
    }
}

impl<T: Serialize> Display for APIResponse<T> {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{}", serde_json::to_string(self).unwrap())
    }
}

impl Display for RequestErrorResponse {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{}", serde_json::to_string(self).unwrap())
    }
}

impl From<RequestErrorResponse> for APIResponse<RequestErrorResponse> {
    fn from(error: RequestErrorResponse) -> Self {
        return APIResponse::new(false, Some(error));
    }
}

impl<T: Serialize> From<Option<T>> for APIResponse<T> {
    /// If the value is None it will create a 404 response
    /// If the value is Some it will set Success to True and the data is provided
    fn from(value: Option<T>) -> Self {
        return if value.is_none() {
            APIResponse {
                success: true,
                data: None,
                status_code: Some(404),
            }
        } else {
            APIResponse::<T>::new(true, value)
        };
    }
}


impl<T: Serialize> APIResponse<T> {
    pub fn new(success: bool, data: Option<T>) -> APIResponse<T> {
        return APIResponse {
            success,
            data,
            status_code: None,
        };
    }
    pub fn error(&self, status: StatusCode) -> SiteResponse {
        return Ok(HttpResponse::Ok()
            .status(status)
            .content_type("application/json")
            .body(serde_json::to_string(self).unwrap()));
    }
    pub fn respond(self, _req: &HttpRequest) -> SiteResponse {
        let i = self.status_code.unwrap_or(200);
        let result = HttpResponse::Ok()
            .status(StatusCode::from_u16(i).unwrap_or(StatusCode::OK))
            .content_type("application/json")
            .body(serde_json::to_string(&self).unwrap());
        return Ok(result);
    }
    pub fn respond_new<S: Into<APIResponse<T>>>(response: S, _req: &HttpRequest) -> SiteResponse {
        let response = response.into();
        let i = response.status_code.unwrap_or(200);
        let result = HttpResponse::Ok()
            .status(StatusCode::from_u16(i).unwrap_or(StatusCode::OK))
            .content_type("application/json")
            .body(serde_json::to_string(&response).unwrap());
        return Ok(result);
    }
}

impl<T: Serialize> Responder for APIResponse<T> {
    type Error = RequestError;
    type Future = futures_util::future::Ready<Result<HttpResponse, RequestError>>;

    fn respond_to(self, _req: &HttpRequest) -> Self::Future {
        return futures_util::future::ok(self.respond(_req).unwrap());
    }
}

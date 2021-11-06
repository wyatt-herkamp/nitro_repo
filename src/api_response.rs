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
}

impl<T: Serialize> Responder for APIResponse<T> {
    type Error = RequestError;
    type Future = futures_util::future::Ready<Result<HttpResponse, RequestError>>;

    fn respond_to(self, _req: &HttpRequest) -> Self::Future {
        return futures_util::future::ok(self.respond(_req).unwrap());
    }
}

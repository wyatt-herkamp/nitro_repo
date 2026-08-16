use std::borrow::Cow;

use axum::response::IntoResponse;
use thiserror::Error;

use crate::utils::{IntoErrorResponse, ResponseBuilder, api_error_response::APIErrorResponse};

#[derive(Debug, Error, Clone, Copy)]
#[error("{0} is missing from extensions")]
pub struct MissingInternelExtension(pub &'static str);
impl IntoErrorResponse for MissingInternelExtension {
    fn into_response_boxed(self: Box<Self>) -> axum::response::Response {
        self.into_response()
    }
}
impl IntoResponse for MissingInternelExtension {
    fn into_response(self) -> axum::response::Response {
        let message: APIErrorResponse<&'static str, ()> = APIErrorResponse {
            message: Cow::Owned(self.to_string()),
            details: Some(self.0),
            error: None,
        };
        ResponseBuilder::internal_server_error()
            .error_reason(format!("Missing Internal Extension {}", self.0))
            .json(&message)
    }
}

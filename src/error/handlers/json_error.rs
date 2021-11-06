
use actix_web::error::JsonPayloadError;
use actix_web::web::JsonConfig;
use actix_web::HttpRequest;
use crate::api_response::{APIResponse, RequestErrorResponse};

pub fn json_config() -> JsonConfig {
    JsonConfig::default().error_handler(handle)
}

pub fn handle(payload: JsonPayloadError, _request: &HttpRequest) -> actix_web::Error {
    return match payload {
        JsonPayloadError::Overflow => actix_web::error::ErrorBadRequest(
            APIResponse::from(RequestErrorResponse::new("Json Overflow", "INTERNAL")),
        ),
        JsonPayloadError::ContentType => actix_web::error::ErrorBadRequest(
            APIResponse::from(RequestErrorResponse::new("Json Bad Content Type", "CONTENT_TYPE")),
        ),
        JsonPayloadError::Deserialize(_) => actix_web::error::ErrorBadRequest(
            APIResponse::from(RequestErrorResponse::new("Invalid Json", "JSON")),
        ),
        JsonPayloadError::Payload(_) => actix_web::error::ErrorBadRequest(
            APIResponse::from(RequestErrorResponse::new("BAD PAYLOAD", "PAYLOAD")),
        ),
    };
}

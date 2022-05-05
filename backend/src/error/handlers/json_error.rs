use actix_web::error::{ErrorBadRequest, JsonPayloadError};
use actix_web::http::StatusCode;
use actix_web::web::JsonConfig;
use actix_web::HttpRequest;
use log::trace;

pub fn json_config() -> JsonConfig {
    JsonConfig::default().error_handler(handle)
}

pub fn handle(payload: JsonPayloadError, _request: &HttpRequest) -> actix_web::Error {
    trace!("JSON Error: {}", payload);
    ErrorBadRequest(format!("Bad Json Payload {}", payload.to_string()))
}

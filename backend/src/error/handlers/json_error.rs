use actix_web::{
    error::{ErrorBadRequest, JsonPayloadError},
    web::JsonConfig,
    HttpRequest,
};
use log::warn;

pub fn json_config() -> JsonConfig {
    JsonConfig::default().error_handler(handle)
}

pub fn handle(payload: JsonPayloadError, request: &HttpRequest) -> actix_web::Error {
    warn!("JSON Error: {}. Path: {}", payload, request.path());
    ErrorBadRequest(format!("Bad Json Payload {}", payload))
}

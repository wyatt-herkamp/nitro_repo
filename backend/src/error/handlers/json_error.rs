use actix_web::error::{ErrorBadRequest, JsonPayloadError};
use actix_web::web::JsonConfig;
use actix_web::HttpRequest;
use log::{as_serde, warn};

pub fn json_config() -> JsonConfig {
    JsonConfig::default().error_handler(handle)
}

pub fn handle(payload: JsonPayloadError, request: &HttpRequest) -> actix_web::Error {
    warn!("JSON Error: {}. Path: {}", payload, request.path());
    ErrorBadRequest(format!("Bad Json Payload {}", payload))
}

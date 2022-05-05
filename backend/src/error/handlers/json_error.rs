use crate::api_response::APIResponse;
use actix_web::error::JsonPayloadError;
use actix_web::http::StatusCode;
use actix_web::web::JsonConfig;
use actix_web::HttpRequest;
use log::trace;

pub fn json_config() -> JsonConfig {
    JsonConfig::default().error_handler(handle)
}

pub fn handle(payload: JsonPayloadError, _request: &HttpRequest) -> actix_web::Error {
    trace!("JSON Error: {}", payload);
    ("Json Bad Content Type", StatusCode::BAD_REQUEST).into()
}

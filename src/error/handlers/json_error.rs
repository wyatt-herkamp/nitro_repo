use crate::error::request_error::RequestError;
use actix_web::error::JsonPayloadError;
use actix_web::web::JsonConfig;
use actix_web::HttpRequest;

pub fn json_config() -> JsonConfig {
    JsonConfig::default().error_handler(handle)
}
pub fn handle(payload: JsonPayloadError, _request: &HttpRequest) -> actix_web::Error {
    return match payload {
        JsonPayloadError::Overflow => actix_web::error::ErrorBadRequest(
            RequestError::MissingArgument("Overflow".into())
                .to_json_response()
                .value,
        ),
        JsonPayloadError::ContentType => actix_web::error::ErrorBadRequest(
            RequestError::MissingArgument("Invalid Content Type".into())
                .to_json_response()
                .value,
        ),
        JsonPayloadError::Deserialize(serde) => actix_web::error::ErrorBadRequest(
            RequestError::MissingArgument(format!("Invalid Json {}", serde.to_string()).into())
                .to_json_response()
                .value,
        ),
        JsonPayloadError::Payload(payload) => actix_web::error::ErrorBadRequest(
            RequestError::MissingArgument(format!("Bad payload {}", payload.to_string()).into())
                .to_json_response()
                .value,
        ),
    };
}

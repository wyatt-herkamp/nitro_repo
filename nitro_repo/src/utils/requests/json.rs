use std::borrow::Cow;

use axum::{
    body::Bytes,
    extract::{FromRequest, Request},
    response::{IntoResponse, Response},
};
use derive_more::{AsRef, Deref, From};
use http::{HeaderMap, header::CONTENT_TYPE};
use serde::de::DeserializeOwned;
use thiserror::Error;
use tracing::{Level, event};

use crate::utils::{ErrorReason, api_error_response::APIErrorResponse, builder::ResponseBuilder};

/// The same as [axum::Json] but with logging of errors and formatted error responses
#[derive(Debug, Clone, Copy, Default, Deref, AsRef, From)]
pub struct JsonBody<T>(pub T);

impl<T, S> FromRequest<S> for JsonBody<T>
where
    T: DeserializeOwned,
    S: Send + Sync,
{
    type Rejection = JsonBodyRejection;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        if is_json_content_type(req.headers()) {
            let bytes = Bytes::from_request(req, state).await.map_err(|err| {
                event!(Level::DEBUG, error = ?err, "Failed to buffer the request body");
                JsonBodyRejection::BytesRejection
            })?;
            Self::from_bytes(&bytes)
        } else {
            event!(
                Level::TRACE,
                "Expected request with `Content-Type: application/json`"
            );
            Err(JsonBodyRejection::MissingJsonContentType)
        }
    }
}

fn is_json_content_type(headers: &HeaderMap) -> bool {
    let content_type = if let Some(content_type) = headers.get(CONTENT_TYPE) {
        content_type
    } else {
        event!(Level::TRACE, "Missing `Content-Type` header");
        return false;
    };

    let content_type = if let Ok(content_type) = content_type.to_str() {
        content_type
    } else {
        event!(Level::TRACE, "Failed to parse `Content-Type` header");
        return false;
    };

    let mime = if let Ok(mime) = content_type.parse::<mime::Mime>() {
        mime
    } else {
        event!(Level::TRACE, "Failed to parse `Content-Type` header");
        return false;
    };
    event!(Level::TRACE, content_type = %content_type, "Parsed `Content-Type` header");
    let is_json_content_type = mime.type_() == "application"
        && (mime.subtype() == "json" || mime.suffix().is_some_and(|name| name == "json"));

    is_json_content_type
}

impl<T> JsonBody<T>
where
    T: DeserializeOwned,
{
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, JsonBodyRejection> {
        let deserializer = &mut serde_json::Deserializer::from_slice(bytes);

        let value = match serde_path_to_error::deserialize(deserializer) {
            Ok(value) => value,
            Err(err) => {
                let path = err.path().clone();
                let inner = err.into_inner();
                let classification = inner.classify();
                event!(Level::DEBUG, error = %inner, classification = ?classification, "Failed to deserialize the JSON body into the target type");
                let rejection = match inner.classify() {
                    serde_json::error::Category::Data => {
                        JsonBodyRejection::JsonDataError(inner, path)
                    }
                    serde_json::error::Category::Syntax | serde_json::error::Category::Eof => {
                        JsonBodyRejection::JsonSyntaxError(inner)
                    }
                    serde_json::error::Category::Io => {
                        // This should never happen because we are deserializing from a slice
                        //  This one is actually reported as an error because. This should never happen
                        event!(Level::ERROR, error = %inner, "Failed to buffer the request body.");
                        JsonBodyRejection::JsonBodyIoError
                    }
                };
                return Err(rejection);
            }
        };

        Ok(JsonBody(value))
    }
}

#[derive(Debug, Error)]
pub enum JsonBodyRejection {
    #[error("Failed to deserialize the JSON body into the target type")]
    JsonDataError(serde_json::Error, serde_path_to_error::Path),
    #[error("Failed to parse the request body as JSON")]
    JsonSyntaxError(serde_json::Error),
    #[error("Expected request with `Content-Type: application/json`")]
    MissingJsonContentType,
    #[error("Failed to buffer the request body")]
    BytesRejection,
    #[error("Failed to buffer the request body")]
    JsonBodyIoError,
}
impl IntoResponse for JsonBodyRejection {
    fn into_response(self) -> Response {
        match self {
            JsonBodyRejection::JsonDataError(err, path) => {
                let reason: ErrorReason =
                    ErrorReason::from(format!("Error Deserializing JSON: {}", err));
                let body = APIErrorResponse {
                    error: Some(err),
                    message: Cow::Borrowed(
                        "Failed to deserialize the JSON body into the target type",
                    ),
                    details: Some(path.to_string()),
                };
                ResponseBuilder::bad_request().extension(reason).json(&body)
            }
            JsonBodyRejection::JsonSyntaxError(err) => {
                let reason: ErrorReason =
                    ErrorReason::from(format!("Error Deserializing JSON: {}", err));

                let body = APIErrorResponse::<(), _> {
                    error: Some(err),
                    message: Cow::Borrowed(
                        "Failed to deserialize the JSON body into the target type",
                    ),
                    details: None,
                };
                ResponseBuilder::bad_request().extension(reason).json(&body)
            }
            JsonBodyRejection::MissingJsonContentType => {
                let body = APIErrorResponse::<(), ()> {
                    error: None,
                    message: Cow::Borrowed(
                        "Expected request with `Content-Type: application/json`",
                    ),
                    details: None,
                };
                ResponseBuilder::unsupported_media_type()
                    .extension(ErrorReason::from("Expected application/json"))
                    .json(&body)
            }
            other => {
                let error_reason = ErrorReason::from(other.to_string());
                let body = APIErrorResponse::<(), _> {
                    error: Some(other),
                    message: Cow::Borrowed(
                        "Failed to deserialize the JSON body into the target type",
                    ),
                    details: None,
                };
                ResponseBuilder::bad_request()
                    .extension(error_reason)
                    .json(&body)
            }
        }
    }
}

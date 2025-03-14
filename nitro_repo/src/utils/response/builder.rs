use std::any::Any;

use axum::response::{IntoResponse, Response};
use error::ResponseBuildError;
use http::{HeaderName, HeaderValue, StatusCode, header::CONTENT_TYPE};
use mime::Mime;

use crate::utils::other::JSON_MEDIA_TYPE;
pub mod error;
use super::ErrorReason;

macro_rules! new_response_builder {
    (
       $( $fn_name:ident => $status:ident),*
    ) => {
        $(
            /// Create a new response builder with the [StatusCode] $status
            pub fn $fn_name() -> Self {
                Self(Response::builder().status(StatusCode::$status))
            }
        )*
    };
}
pub struct ResponseBuilder(pub http::response::Builder);
/// When a body method is called. It converts it to a response. So if you return a response builder it will be empty body
impl IntoResponse for ResponseBuilder {
    fn into_response(self) -> Response {
        self.empty()
    }
}
impl Default for ResponseBuilder {
    fn default() -> Self {
        Self(Response::builder().status(StatusCode::NO_CONTENT))
    }
}
impl ResponseBuilder {
    pub fn status(self, status: StatusCode) -> Self {
        Self(self.0.status(status))
    }
    new_response_builder!(
        ok => OK,
        created => CREATED,
        no_content => NO_CONTENT,
        bad_request => BAD_REQUEST,
        not_found => NOT_FOUND,
        conflict => CONFLICT,
        unauthorized => UNAUTHORIZED,
        forbidden => FORBIDDEN,
        internal_server_error => INTERNAL_SERVER_ERROR,
        unsupported_media_type => UNSUPPORTED_MEDIA_TYPE
    );
    pub fn content_type(self, content_type: Mime) -> Self {
        self.header(CONTENT_TYPE, content_type.to_string())
    }
    pub fn error_reason(self, reason: impl Into<ErrorReason>) -> Self {
        Self(self.0.extension(reason.into()))
    }
    pub fn extension<T>(self, extension: T) -> Self
    where
        T: Clone + Any + Send + Sync + 'static,
    {
        Self(self.0.extension(extension))
    }
    /// Sets the body if it returns an error it will return a [ResponseBuildError]
    pub fn body_or_err(
        self,
        body: impl Into<axum::body::Body>,
    ) -> Result<Response, ResponseBuildError> {
        let body = body.into();
        self.0.body(body).map_err(ResponseBuildError::HttpError)
    }
    /// Sets the body if it returns an error it will return a [ResponseBuildError]
    pub fn body(self, body: impl Into<axum::body::Body>) -> Response {
        match self.body_or_err(body) {
            Ok(ok) => ok,
            Err(err) => err.into_response(),
        }
    }
    /// Empty body
    pub fn empty(self) -> Response {
        self.body(axum::body::Body::empty())
    }
    pub fn header<K, V>(self, key: K, value: V) -> Self
    where
        K: TryInto<HeaderName>,
        <K as TryInto<HeaderName>>::Error: Into<http::Error>,
        V: TryInto<HeaderValue>,
        <V as TryInto<HeaderValue>>::Error: Into<http::Error>,
    {
        Self(self.0.header(key, value))
    }

    /// Serialize the data to JSON and return a response or an error
    pub fn json_or_err<T: serde::Serialize>(
        self,
        data: &T,
    ) -> Result<Response, ResponseBuildError> {
        let body = serde_json::to_vec(data)?;
        self.header(CONTENT_TYPE, JSON_MEDIA_TYPE).body_or_err(body)
    }
    /// Serialize the data to JSON and return a response
    pub fn json<T: serde::Serialize>(self, data: &T) -> Response {
        match self.json_or_err(data) {
            Ok(ok) => ok,
            Err(err) => err.into_response(),
        }
    }
    /// Checks if the data is present and returns a JSON response or a not found response
    pub fn json_or_not_found<T: serde::Serialize>(self, data: &Option<T>) -> Response {
        match data {
            Some(data) => self.json(data),
            None => self.status(StatusCode::NOT_FOUND).empty(),
        }
    }
    pub fn html_or_err(self, html: impl Into<Vec<u8>>) -> Result<Response, ResponseBuildError> {
        self.content_type(mime::TEXT_HTML_UTF_8)
            .body_or_err(html.into())
    }
    pub fn html(self, html: impl Into<Vec<u8>>) -> Response {
        match self.html_or_err(html) {
            Ok(ok) => ok,
            Err(err) => err.into_response(),
        }
    }
}

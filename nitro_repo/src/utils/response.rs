pub mod builder;

use std::{borrow::Cow, error::Error};

pub use builder::ResponseBuilder;
use derive_more::From;
pub mod api_error_response;
pub mod conflict;
pub trait IntoErrorResponse: Error + Send + Sync {
    /// Converts the error into a response
    ///
    /// It must be of type of Box<Self> to allow for dynamic dispatch
    fn into_response_boxed(self: Box<Self>) -> axum::response::Response;
    #[inline(always)]
    fn json_error_response(self: Box<Self>) -> Option<axum::response::Response> {
        None
    }
    #[inline(always)]
    fn supports_json_error_response(&self) -> bool {
        false
    }

    fn error_reason(&self) -> ErrorReason {
        ErrorReason::from(self.to_string())
    }
}

#[derive(Debug, Clone, From)]
pub struct ErrorReason {
    pub reason: Cow<'static, str>,
}
impl From<String> for ErrorReason {
    fn from(reason: String) -> Self {
        Self {
            reason: Cow::Owned(reason),
        }
    }
}
impl From<&'static str> for ErrorReason {
    fn from(reason: &'static str) -> Self {
        Self {
            reason: Cow::Borrowed(reason),
        }
    }
}

use std::{
    borrow::Cow,
    fmt::{Debug, Display},
};

use serde::{Serialize, Serializer, ser::SerializeMap};
use utoipa::ToSchema;

/// The documented shape of an error body.
///
/// A separate, non-generic type on purpose. utoipa registers a generic schema under a name that
/// includes its type arguments (`APIErrorResponse_String_String`), while `ToSchema::name()` returns
/// the bare `APIErrorResponse` — so anything building a `$ref` from `name()` pointed at a component
/// that was not in the document, and every consumer that resolves references rejected it. This is
/// what gets registered; `APIErrorResponse` below is what the code actually constructs, and the two
/// serialize identically (see `Serialize` for `APIErrorResponse`).
#[derive(Debug, ToSchema)]
#[schema(as = APIErrorResponse)]
pub struct APIErrorResponseSchema {
    /// The message to display to the user
    pub message: String,
    /// The error that caused the issue, if any
    #[schema(nullable)]
    pub error: Option<String>,
    /// Additional details about the error, if any
    #[schema(value_type = Option<nr_core::utils::utopia::AnyType>, nullable)]
    pub details: Option<serde_json::Value>,
}

#[derive(Debug, ToSchema)]
pub struct APIErrorResponse<D = (), E = Box<dyn Debug>> {
    /// The message to display to the user
    pub message: Cow<'static, str>,
    /// The error that caused the issue if any
    #[schema(value_type = Option<String>, nullable)]
    pub error: Option<E>,
    /// Additional details about the error if any
    pub details: Option<D>,
}
impl<D, E: Debug> Serialize for APIErrorResponse<D, E>
where
    D: Serialize,
    E: Debug,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map_serializer = serializer.serialize_map(Some(3))?;
        map_serializer.serialize_entry("message", &self.message)?;
        if let Some(error) = &self.error {
            map_serializer.serialize_entry("error", &format!("{:?}", error))?;
        }
        if let Some(details) = &self.details {
            map_serializer.serialize_entry("details", details)?;
        }
        map_serializer.end()
    }
}
impl<T, E> Default for APIErrorResponse<T, E> {
    fn default() -> Self {
        APIErrorResponse {
            message: Cow::Borrowed("Unknown Error"),
            error: None,
            details: None,
        }
    }
}
impl From<&'static str> for APIErrorResponse {
    fn from(message: &'static str) -> Self {
        APIErrorResponse {
            message: Cow::Borrowed(message),
            error: None,
            details: None,
        }
    }
}

impl<E> From<(E, &'static str)> for APIErrorResponse<(), E>
where
    E: Debug + 'static,
{
    fn from((error, message): (E, &'static str)) -> Self {
        APIErrorResponse {
            message: Cow::Borrowed(message),
            error: Some(error),
            details: None,
        }
    }
}

impl<T, E> Display for APIErrorResponse<T, E>
where
    T: Debug,
    E: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self {
            message,
            error,
            details,
        } = self;
        writeln!(f, "{message}")?;
        if let Some(error) = error {
            writeln!(f, "Error: {:?}", error)?;
        }
        if let Some(details) = details {
            writeln!(f, "Details: {:?}", details)?;
        }
        Ok(())
    }
}

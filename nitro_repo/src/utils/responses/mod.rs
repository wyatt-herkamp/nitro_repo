mod conflict;
use std::{
    borrow::Cow,
    fmt::{Debug, Display},
};

pub use conflict::*;
use serde::{ser::SerializeMap, Serialize, Serializer};
use utoipa::ToSchema;

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

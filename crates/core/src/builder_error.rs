use derive_builder::UninitializedFieldError;
use thiserror::Error;

#[derive(Debug, Clone, Error)]
pub enum BuilderError {
    #[error("Uninitialized Field: {0}.")]
    UninitializedField(&'static str),
    #[error("Invalid Field: {0}.")]
    InvalidField(String),
}

impl From<UninitializedFieldError> for BuilderError {
    fn from(err: UninitializedFieldError) -> Self {
        BuilderError::UninitializedField(err.field_name())
    }
}

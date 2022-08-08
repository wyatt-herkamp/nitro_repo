use this_actix_error::ActixError;
use thiserror::Error;

#[derive(Debug, Error, ActixError)]
pub enum NPMError {
    #[status_code(400)]
    #[error("Invalid NPM Command `{0}`")]
    InvalidCommand(String),
}

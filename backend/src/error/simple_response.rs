use this_actix_error::ActixError;
use thiserror::Error;

#[derive(Error, Debug, ActixError)]
pub enum SimpleResponse {
    #[status_code(NOT_FOUND)]
    #[error("Not Found `{0}`")]
    BadStorageName(String),
    #[status_code(NOT_FOUND)]
    #[error("Not Found `{0}`")]
    BadRepositoryName(String),

}
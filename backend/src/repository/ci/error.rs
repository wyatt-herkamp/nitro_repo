use crate::repository::settings::Policy;

use this_actix_error::ActixError;
use thiserror::Error;

#[derive(Debug, Error, ActixError)]
pub enum CIError {
    #[status_code(400)]
    #[error("{0}")]
    BadRequest(&'static str),

}

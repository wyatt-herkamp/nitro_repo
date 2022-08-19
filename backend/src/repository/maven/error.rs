use crate::repository::settings::Policy;

use this_actix_error::ActixError;
use thiserror::Error;

#[derive(Debug, Error, ActixError)]
pub enum MavenError {
    #[status_code(400)]
    #[error("{0} Only Repository")]
    PolicyError(Policy),
    #[status_code(400)]
    #[error("Unable to Parse Pom File")]
    PomError,
}

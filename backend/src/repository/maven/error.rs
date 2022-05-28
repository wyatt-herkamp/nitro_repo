use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub struct MavenError(String);

impl Display for MavenError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Error for MavenError {}

impl From<&str> for MavenError {
    fn from(err: &str) -> MavenError {
        MavenError(err.to_string())
    }
}

impl From<String> for MavenError {
    fn from(err: String) -> MavenError {
        MavenError(err)
    }
}

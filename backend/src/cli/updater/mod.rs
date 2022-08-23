pub mod one_one;

use thiserror::Error;

#[derive(Error, Clone, Debug)]
pub enum UpdateError {
    #[error("{0}")]
    Other(&'static str),
}


pub async fn update(version: impl AsRef<str>) -> Result<(), UpdateError> {
    match version.as_ref() {
        "1.1" => {
            one_one::update().await?;
        }
        _ => {
            return Err(UpdateError::Other("Unsupported version"));
        }
    }
    Ok(())
}

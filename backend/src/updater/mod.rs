#[cfg(feature = "updater_one-one")]
pub mod one_one;

use thiserror::Error;

#[derive(Error, Clone, Debug)]
pub enum UpdateError {}

pub async fn update(_version: String) -> Result<(), UpdateError> {
    Ok(())
}

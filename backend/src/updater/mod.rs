#[cfg(feature = "updater_one-one")]
pub mod one_one;

use thiserror::Error;

#[derive(Error, Clone, Debug)]
pub enum UpdateError {}

pub async fn update(version: String) -> Result<(), UpdateError> {
    return Ok(());
}
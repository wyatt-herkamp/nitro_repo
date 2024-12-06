#[cfg(feature = "migrations")]
pub mod migration;
pub mod project;
pub mod repository;
pub mod storage;
pub mod user;
pub type DateTime = chrono::DateTime<chrono::FixedOffset>;
mod config;
pub mod stages;
pub use config::*;

#[derive(thiserror::Error, Debug)]
pub enum DBError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
    #[error(transparent)]
    Migration(#[from] sqlx::migrate::MigrateError),
    #[error("{0}")]
    Other(&'static str),
    #[error("Invalid host must be in the format host:port got `{0}`")]
    InvalidHost(String),
}

pub mod entities;
#[cfg(feature = "migrations")]
pub mod migration;
pub type DateTime = chrono::DateTime<chrono::FixedOffset>;
mod config;

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
pub type DBResult<T> = Result<T, DBError>;

pub mod prelude {
    pub use super::{DBError, DBResult};
    pub use chrono::{DateTime, FixedOffset, Local, NaiveDate};
    pub use pg_extended_sqlx_queries::prelude::*;
    pub use sqlx::{FromRow, PgPool, Postgres, QueryBuilder, postgres::PgRow, prelude::*};
    pub use tracing::{debug, error, info, instrument, trace, warn};
}

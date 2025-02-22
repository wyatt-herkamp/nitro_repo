use nr_macros::{NuType, SerdeViaStr};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sqlx::prelude::Type;
use strum::{EnumIs, EnumIter};
use thiserror::Error;
use tracing::instrument;
use utoipa::ToSchema;

use crate::utils::validations::{self};
pub mod browse;
pub mod config;
pub mod project;
pub mod proxy_url;
#[derive(
    Debug,
    Clone,
    Copy,
    Serialize,
    Deserialize,
    PartialEq,
    Eq,
    Default,
    EnumIter,
    ToSchema,
    EnumIs,
    Type,
)]
#[sqlx(type_name = "VARCHAR")]
pub enum Visibility {
    /// Completely public anyone can read to this repository
    #[default]
    Public,
    /// Private. Only users with the correct permissions can read this repository
    Private,
    /// Hidden. You can read this repository but indexing will be disabled
    Hidden,
}
#[derive(
    Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default, EnumIter, JsonSchema, EnumIs,
)]
pub enum Policy {
    Release,
    Snapshot,
    #[default]
    Mixed,
}
#[derive(Debug, Error)]
pub enum InvalidRepositoryName {
    #[error("Repository name is too short, must be at least 3 got {0} characters")]
    TooShort(usize),
    #[error("Repository name is too long, must be less than 32 got {0} characters")]
    TooLong(usize),
    #[error(
        "Repository name contains invalid character `{0}`. Repository Names can only contain letters, numbers, `_`, and `-`"
    )]
    InvalidCharacter(char),
}
#[derive(Debug, Type, Clone, Default, SerdeViaStr, NuType)]
#[sqlx(transparent)]
pub struct RepositoryName(String);
validations::schema_for_new_type_str!(RepositoryName, pattern = r#"^([a-zA-Z0-9_\-]{3,32}$)"#);
validations::convert_traits_to_new!(RepositoryName, InvalidRepositoryName);
impl RepositoryName {
    #[instrument(name = "RepositoryName::new")]
    pub fn new(repository_name: String) -> Result<Self, InvalidRepositoryName> {
        if repository_name.len() < 3 {
            return Err(InvalidRepositoryName::TooShort(repository_name.len()));
        }
        if repository_name.len() > 32 {
            return Err(InvalidRepositoryName::TooLong(repository_name.len()));
        }
        if let Some(bad_char) = repository_name
            .chars()
            .find(|c| !validations::valid_name_char(*c))
        {
            return Err(InvalidRepositoryName::InvalidCharacter(bad_char));
        }
        Ok(Self(repository_name))
    }
}

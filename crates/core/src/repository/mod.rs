use std::fmt::Display;

use derive_more::derive::{AsRef, Deref, Into};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sqlx::prelude::Type;
use strum::{EnumIs, EnumIter};
use thiserror::Error;
use tracing::instrument;

use crate::utils::validations;
pub mod config;
pub mod project;
#[derive(
    Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default, EnumIter, JsonSchema, EnumIs,
)]
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
    #[default]
    Release,
    Snapshot,
    Mixed,
}
#[derive(Debug, Error)]
pub enum InvalidRepositoryName {
    #[error("Repository name is too short, must be at least 3 got {0} characters")]
    TooShort(usize),
    #[error("Repository name is too long, must be less than 32 got {0} characters")]
    TooLong(usize),
    #[error("Repository name contains invalid character `{0}`. Repository Names can only contain letters, numbers, `_`, and `-`")]
    InvalidCharacter(char),
}
#[derive(Debug, Type, Deref, AsRef, Clone, PartialEq, Eq, Default, Into)]
#[sqlx(transparent)]
#[as_ref(forward)]
pub struct RepositoryName(String);
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
impl Display for RepositoryName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

validations::from_impls!(RepositoryName, InvalidRepositoryName);

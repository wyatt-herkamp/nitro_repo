use std::fmt::Debug;

use digestible::Digestible;
use schemars::Schema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::PgPool;
use thiserror::Error;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::database::repository::DBRepositoryConfig;
pub mod project;
pub mod repository_page;
#[derive(Debug, Error)]
pub enum RepositoryConfigError {
    #[error("Invalid Config: {0}")]
    InvalidConfig(&'static str),
    #[error("Invalid Config: {0}")]
    SerdeError(#[from] serde_json::Error),
    #[error("Invalid Change: {0}")]
    InvalidChange(&'static str, &'static str),
}
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Digestible)]
#[derive(Default)]
pub struct ConfigDescription {
    pub name: &'static str,
    pub description: Option<&'static str>,
    pub documentation_link: Option<&'static str>,
    pub has_public_view: bool,
}
/// A Config Type is a type that should be zero sized and should be used to validate and define the layout of a config for a repository
///
/// An array of these will be created at start of the program and can be retrieved to validate and create configs for a repository
pub trait RepositoryConfigType: Send + Sync + Debug {
    /// The config name. This is used to identify the config type in the database
    fn get_type(&self) -> &'static str;

    fn get_type_static() -> &'static str
    where
        Self: Sized;

    fn get_description(&self) -> ConfigDescription {
        ConfigDescription {
            name: self.get_type(),
            ..Default::default()
        }
    }
    /// Sanitizes the config for public view.
    ///
    /// By default this function returns None which will mean the config is not shown to the public
    #[inline(always)]
    fn sanitize_for_public_view(&self, _: Value) -> Result<Option<Value>, RepositoryConfigError> {
        Ok(None)
    }
    /// Validate the config. If the config is invalid this function should return an error
    fn validate_config(&self, config: Value) -> Result<(), RepositoryConfigError>;
    /// If part of the config cannot be changed this function should return an error
    fn validate_change(&self, _old: Value, new: Value) -> Result<(), RepositoryConfigError> {
        self.validate_config(new)
    }
    /// Get the default config. Errors are usually a bug in the code
    fn default(&self) -> Result<Value, RepositoryConfigError>;
    /// Schema for the config

    fn schema(&self) -> Option<Schema> {
        None
    }
}
pub async fn get_repository_config_or_default<
    T: RepositoryConfigType,
    D: for<'a> Deserialize<'a> + Unpin + Send + 'static + Default,
>(
    repository: Uuid,
    db: &PgPool,
) -> Result<DBRepositoryConfig<D>, sqlx::Error> {
    DBRepositoryConfig::<D>::get_config(repository, T::get_type_static(), db)
        .await
        .map(|x| x.unwrap_or_default())
}
pub type DynRepositoryConfigType = Box<dyn RepositoryConfigType>;

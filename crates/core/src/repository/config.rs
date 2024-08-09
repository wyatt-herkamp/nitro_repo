use std::fmt::Debug;

use schemars::{schema_for, JsonSchema};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::PgPool;
use thiserror::Error;
use uuid::Uuid;

use crate::database::repository::DBRepositoryConfig;

use super::{Policy, Visibility};
pub mod frontend;
#[derive(Debug, Error)]
pub enum RepositoryConfigError {
    #[error("Invalid Config: {0}")]
    InvalidConfig(&'static str),
    #[error("Invalid Config: {0}")]
    SerdeError(#[from] serde_json::Error),
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
    /// Validate the config. If the config is invalid this function should return an error
    fn validate_config(&self, config: Value) -> Result<(), RepositoryConfigError>;
    /// If part of the config cannot be changed this function should return an error
    fn validate_change(&self, _old: Value, new: Value) -> Result<(), RepositoryConfigError> {
        self.validate_config(new)
    }
    /// Get the default config. Errors are usually a bug in the code
    fn default(&self) -> Result<Value, RepositoryConfigError>;
    /// Schema for the config

    fn schema(&self) -> Option<schemars::schema::RootSchema> {
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
#[derive(Debug, Clone, Serialize, Deserialize, Default, JsonSchema)]
pub struct SecurityConfig {
    /// If the repository requires an auth token to be used
    pub must_use_auth_token_for_push: bool,
    /// visibility of the repository
    pub visibility: Visibility,
}
#[derive(Debug, Clone, Copy, Default)]
pub struct SecurityConfigType;
impl RepositoryConfigType for SecurityConfigType {
    fn get_type(&self) -> &'static str {
        "security"
    }

    fn validate_config(&self, config: Value) -> Result<(), RepositoryConfigError> {
        let _config: SecurityConfig = serde_json::from_value(config)?;
        Ok(())
    }

    fn default(&self) -> Result<Value, RepositoryConfigError> {
        Ok(serde_json::to_value(SecurityConfig::default())?)
    }

    fn schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schema_for!(SecurityConfig))
    }

    fn get_type_static() -> &'static str
    where
        Self: Sized,
    {
        "security"
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, JsonSchema)]
pub struct PushRulesConfig {
    /// The push policy. Rather it allows snapshots, stages, or both
    pub push_policy: Policy,
    /// If yanking is allowed
    pub yanking_allowed: bool,
    /// If overwriting is allowed
    pub allow_overwrite: bool,
}
#[derive(Debug, Clone, Copy, Default)]

pub struct PushRulesConfigType;
impl RepositoryConfigType for PushRulesConfigType {
    fn get_type(&self) -> &'static str {
        "push_rules"
    }

    fn validate_config(&self, config: Value) -> Result<(), RepositoryConfigError> {
        let _config: PushRulesConfig = serde_json::from_value(config)?;
        Ok(())
    }

    fn default(&self) -> Result<Value, RepositoryConfigError> {
        Ok(serde_json::to_value(PushRulesConfig::default())?)
    }

    fn schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schema_for!(PushRulesConfig))
    }

    fn get_type_static() -> &'static str
    where
        Self: Sized,
    {
        "push_rules"
    }
}

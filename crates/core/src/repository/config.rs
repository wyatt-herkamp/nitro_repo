use std::fmt::Debug;

use digestible::Digestible;
use schemars::{schema_for, JsonSchema, Schema};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::PgPool;
use thiserror::Error;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::database::repository::DBRepositoryConfig;
pub mod project;
use super::Policy;
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
pub struct ConfigDescription {
    pub name: &'static str,
    pub description: Option<&'static str>,
    pub documentation_link: Option<&'static str>,
    pub has_public_view: bool,
}
impl Default for ConfigDescription {
    fn default() -> Self {
        ConfigDescription {
            name: "",
            description: None,
            documentation_link: None,
            has_public_view: false,
        }
    }
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
    /// Sanitizes the config for public view. By default this function returns None which will mean the config is not shown to the public
    #[inline(always)]
    fn sanitize_for_public_view(&self, _: Value) -> Option<Value> {
        None
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
#[derive(Debug, Clone, Serialize, Deserialize, Default, JsonSchema)]
#[serde(default)]

pub struct SecurityConfig {
    #[schemars(title = "Require Auth Token for Push")]
    /// If the repository requires an auth token to be used
    pub must_use_auth_token_for_push: bool,
}
#[derive(Debug, Clone, Copy, Default)]
pub struct SecurityConfigType;
impl RepositoryConfigType for SecurityConfigType {
    fn get_type(&self) -> &'static str {
        "security"
    }
    fn get_description(&self) -> ConfigDescription {
        ConfigDescription {
            name: "Security",
            description: Some("Security settings for the repository"),
            documentation_link: None,
            ..Default::default()
        }
    }

    fn validate_config(&self, config: Value) -> Result<(), RepositoryConfigError> {
        let _config: SecurityConfig = serde_json::from_value(config)?;
        Ok(())
    }

    fn default(&self) -> Result<Value, RepositoryConfigError> {
        Ok(serde_json::to_value(SecurityConfig::default())?)
    }

    fn schema(&self) -> Option<Schema> {
        Some(schema_for!(SecurityConfig))
    }

    fn get_type_static() -> &'static str
    where
        Self: Sized,
    {
        "security"
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(default)]
pub struct PushRulesConfig {
    #[schemars(title = "Push Policy")]
    /// The push policy. Rather it allows snapshots, stages, or both
    pub push_policy: Policy,
    /// If yanking is allowed
    #[schemars(title = "Yanking Allowed")]
    pub yanking_allowed: bool,
    /// If overwriting is allowed
    #[schemars(title = "Allow Overwrite")]
    pub allow_overwrite: bool,
    /// If a project exists the user must be a member of the project to push.
    #[schemars(title = "Project Members can only push")]
    pub must_be_project_member: bool,
    #[schemars(title = "Require Nitro Deploy")]
    pub require_nitro_deploy: bool,
}
impl Default for PushRulesConfig {
    fn default() -> Self {
        Self {
            push_policy: Default::default(),
            yanking_allowed: true,
            allow_overwrite: true,
            must_be_project_member: Default::default(),
            require_nitro_deploy: false,
        }
    }
}
#[derive(Debug, Clone, Copy, Default)]

pub struct PushRulesConfigType;
impl RepositoryConfigType for PushRulesConfigType {
    fn get_type(&self) -> &'static str {
        "push_rules"
    }
    fn get_description(&self) -> ConfigDescription {
        ConfigDescription {
            name: "Push Rules",
            description: Some("Rules for pushing to the repository"),
            documentation_link: None,
            ..Default::default()
        }
    }
    fn validate_config(&self, config: Value) -> Result<(), RepositoryConfigError> {
        let _config: PushRulesConfig = serde_json::from_value(config)?;
        Ok(())
    }

    fn default(&self) -> Result<Value, RepositoryConfigError> {
        Ok(serde_json::to_value(PushRulesConfig::default())?)
    }

    fn schema(&self) -> Option<Schema> {
        Some(schema_for!(PushRulesConfig))
    }

    fn get_type_static() -> &'static str
    where
        Self: Sized,
    {
        "push_rules"
    }
}

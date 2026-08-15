use nr_core::repository::{
    Policy,
    config::{ConfigDescription, RepositoryConfigError, RepositoryConfigType},
};
use schemars::{JsonSchema, Schema, schema_for};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::proxy::MavenProxyConfig;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", content = "config")]
pub enum MavenRepositoryConfig {
    Hosted,
    Proxy(MavenProxyConfig),
}
impl MavenRepositoryConfig {
    pub fn is_same_type(&self, other: &MavenRepositoryConfig) -> bool {
        matches!(
            (self, other),
            (MavenRepositoryConfig::Hosted, MavenRepositoryConfig::Hosted)
                | (
                    MavenRepositoryConfig::Proxy(_),
                    MavenRepositoryConfig::Proxy(_)
                )
        )
    }
}
#[derive(Debug, Clone, Default)]
pub struct MavenRepositoryConfigType;
impl RepositoryConfigType for MavenRepositoryConfigType {
    fn get_type(&self) -> &'static str {
        "maven"
    }

    fn get_type_static() -> &'static str
    where
        Self: Sized,
    {
        "maven"
    }
    fn schema(&self) -> Option<schemars::Schema> {
        Some(schema_for!(MavenRepositoryConfig))
    }
    fn validate_config(&self, config: Value) -> Result<(), RepositoryConfigError> {
        // Parsed purely to validate: a config that will not deserialize is not a valid one.
        let _: MavenRepositoryConfig = serde_json::from_value(config)?;
        Ok(())
    }
    fn validate_change(&self, old: Value, new: Value) -> Result<(), RepositoryConfigError> {
        let new: MavenRepositoryConfig = serde_json::from_value(new)?;
        let old: MavenRepositoryConfig = serde_json::from_value(old)?;
        if !old.is_same_type(&new) {
            return Err(RepositoryConfigError::InvalidChange(
                "maven",
                "Cannot change the type of Maven Repository",
            ));
        }
        Ok(())
    }
    fn default(&self) -> Result<Value, RepositoryConfigError> {
        let config = MavenRepositoryConfig::Hosted;
        Ok(serde_json::to_value(config).unwrap())
    }
    fn get_description(&self) -> ConfigDescription {
        ConfigDescription {
            name: "Maven Repository Config",
            description: Some("Handles the type of Maven Repository"),
            documentation_link: None,
            ..Default::default()
        }
    }
}

/// Rules applied to every deploy.
///
/// `require_nitro_deploy` used to live here. Nitro Deploy was never implemented — its types were
/// referenced by nothing, `handle_post` was a `todo!()`, and a PUT carrying the header returned
/// 405 — so setting the flag made a repository **undeployable**: every push got a 400 and there
/// was no combination of client settings that would work. Removed rather than left as a switch
/// whose only effect is to brick a repository. `#[serde(default)]` means existing config rows
/// that still carry the field simply ignore it.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(default)]
pub struct MavenPushRules {
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
    #[schemars(title = "Require Auth Token for Push")]
    /// If the repository requires an auth token to be used
    pub must_use_auth_token_for_push: bool,
}
impl Default for MavenPushRules {
    fn default() -> Self {
        Self {
            push_policy: Default::default(),
            yanking_allowed: true,
            allow_overwrite: true,
            must_be_project_member: Default::default(),
            must_use_auth_token_for_push: false,
        }
    }
}
#[derive(Debug, Clone, Copy, Default)]

pub struct MavenPushRulesConfigType;
impl RepositoryConfigType for MavenPushRulesConfigType {
    fn get_type(&self) -> &'static str {
        Self::get_type_static()
    }

    fn get_type_static() -> &'static str
    where
        Self: Sized,
    {
        "maven_push_rules"
    }
    fn get_description(&self) -> ConfigDescription {
        ConfigDescription {
            name: "Push Rules",
            description: Some("Rules for pushing to a maven repository"),
            documentation_link: None,
            ..Default::default()
        }
    }
    fn validate_config(&self, config: Value) -> Result<(), RepositoryConfigError> {
        let _config: MavenPushRules = serde_json::from_value(config)?;
        Ok(())
    }

    fn default(&self) -> Result<Value, RepositoryConfigError> {
        Ok(serde_json::to_value(MavenPushRules::default())?)
    }

    fn schema(&self) -> Option<Schema> {
        Some(schema_for!(MavenPushRules))
    }
}

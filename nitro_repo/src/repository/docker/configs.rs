use nr_core::repository::config::{ConfigDescription, RepositoryConfigError, RepositoryConfigType};
use schemars::{JsonSchema, schema_for};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Which kind of Docker registry this is.
///
/// Only `Hosted` exists today. A `Proxy` variant carrying an upstream (a pull-through cache of
/// Docker Hub) would be added here the way Maven's was, without a migration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", content = "config")]
pub enum DockerRegistryConfig {
    Hosted,
}

#[derive(Debug, Clone, Default)]
pub struct DockerRegistryConfigType;
impl RepositoryConfigType for DockerRegistryConfigType {
    fn get_type(&self) -> &'static str {
        "docker"
    }

    fn get_type_static() -> &'static str
    where
        Self: Sized,
    {
        "docker"
    }
    fn schema(&self) -> Option<schemars::Schema> {
        Some(schema_for!(DockerRegistryConfig))
    }
    fn validate_config(&self, config: Value) -> Result<(), RepositoryConfigError> {
        let _: DockerRegistryConfig = serde_json::from_value(config)?;
        Ok(())
    }
    fn validate_change(&self, _old: Value, _new: Value) -> Result<(), RepositoryConfigError> {
        Ok(())
    }
    fn default(&self) -> Result<Value, RepositoryConfigError> {
        Ok(serde_json::to_value(DockerRegistryConfig::Hosted)?)
    }
    fn get_description(&self) -> ConfigDescription {
        ConfigDescription {
            name: "Docker Registry Config",
            description: Some("Handles the type of Docker Registry"),
            documentation_link: None,
            ..Default::default()
        }
    }
}

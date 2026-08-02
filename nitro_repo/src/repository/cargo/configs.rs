use nr_core::repository::config::{ConfigDescription, RepositoryConfigError, RepositoryConfigType};
use schemars::{JsonSchema, schema_for};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Which kind of Cargo registry this is.
///
/// Only `Hosted` exists today. It is still a tagged enum rather than a unit struct so a `Proxy`
/// variant carrying an upstream can be added the way Maven's was, without a migration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", content = "config")]
pub enum CargoRegistryConfig {
    Hosted,
}

#[derive(Debug, Clone, Default)]
pub struct CargoRegistryConfigType;
impl RepositoryConfigType for CargoRegistryConfigType {
    fn get_type(&self) -> &'static str {
        "cargo"
    }

    fn get_type_static() -> &'static str
    where
        Self: Sized,
    {
        "cargo"
    }
    fn schema(&self) -> Option<schemars::Schema> {
        Some(schema_for!(CargoRegistryConfig))
    }
    fn validate_config(&self, config: Value) -> Result<(), RepositoryConfigError> {
        let _: CargoRegistryConfig = serde_json::from_value(config)?;
        Ok(())
    }
    fn validate_change(&self, _old: Value, _new: Value) -> Result<(), RepositoryConfigError> {
        Ok(())
    }
    fn default(&self) -> Result<Value, RepositoryConfigError> {
        Ok(serde_json::to_value(CargoRegistryConfig::Hosted)?)
    }
    fn get_description(&self) -> ConfigDescription {
        ConfigDescription {
            name: "Cargo Registry Config",
            description: Some("Handles the type of Cargo Registry"),
            documentation_link: None,
            ..Default::default()
        }
    }
}

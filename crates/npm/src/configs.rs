use nr_core::repository::config::{ConfigDescription, RepositoryConfigError, RepositoryConfigType};
use schemars::{JsonSchema, schema_for};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", content = "config")]
pub enum NPMRegistryConfig {
    Hosted,
}

#[derive(Debug, Clone, Default)]
pub struct NPMRegistryConfigType;
impl RepositoryConfigType for NPMRegistryConfigType {
    fn get_type(&self) -> &'static str {
        "npm"
    }

    fn get_type_static() -> &'static str
    where
        Self: Sized,
    {
        "npm"
    }
    fn schema(&self) -> Option<schemars::Schema> {
        Some(schema_for!(NPMRegistryConfig))
    }
    fn validate_config(&self, config: Value) -> Result<(), RepositoryConfigError> {
        // Parsed purely to validate: a config that will not deserialize is not a valid one.
        let _: NPMRegistryConfig = serde_json::from_value(config)?;
        Ok(())
    }
    /// Accepts any change.
    ///
    /// Note that this overrides the trait default, which would have run `validate_config(new)`.
    /// There is only one npm sub-type, so there is nothing to forbid switching between — but it
    /// does mean an unparseable config is accepted on update where it would be refused on create.
    fn validate_change(&self, _old: Value, _new: Value) -> Result<(), RepositoryConfigError> {
        Ok(())
    }
    fn default(&self) -> Result<Value, RepositoryConfigError> {
        let config = NPMRegistryConfig::Hosted;
        Ok(serde_json::to_value(config).unwrap())
    }
    fn get_description(&self) -> ConfigDescription {
        ConfigDescription {
            name: "NPM Registry Config",
            description: Some("Handles the type of NPM Registry"),
            documentation_link: None,
            ..Default::default()
        }
    }
}

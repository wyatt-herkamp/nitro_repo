use schemars::{JsonSchema, Schema};
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, JsonSchema, Serialize, Deserialize)]
pub struct CommandDefinition {
    pub name: String,
    pub description: String,
    /// Will be prepended with the repository type
    pub key: String,
    pub warn_before_run: Option<String>,
    pub request_schema: Schema,
}

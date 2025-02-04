use schemars::{JsonSchema, Schema, schema_for};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use utoipa::ToSchema;

use super::{RepositoryConfigError, RepositoryConfigType};

#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema, ToSchema)]
pub struct RepositoryPage {
    pub page_type: PageType,
    pub content: Option<String>,
}
#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema, ToSchema)]
pub enum PageType {
    Markdown,
    HTML,
    None,
}
#[derive(Debug, Clone, Copy, Default)]
pub struct RepositoryPageType;
impl RepositoryConfigType for RepositoryPageType {
    fn get_type(&self) -> &'static str {
        "page"
    }
    fn get_description(&self) -> super::ConfigDescription {
        super::ConfigDescription {
            name: "Repository Page",
            description: Some("The page for the repository"),
            documentation_link: None,
            ..Default::default()
        }
    }
    fn validate_config(&self, config: Value) -> Result<(), RepositoryConfigError> {
        let _config: RepositoryPage = serde_json::from_value(config)?;
        Ok(())
    }

    fn default(&self) -> Result<Value, RepositoryConfigError> {
        Ok(serde_json::to_value(RepositoryPage::default())?)
    }

    fn schema(&self) -> Option<Schema> {
        Some(schema_for!(RepositoryPage))
    }

    fn get_type_static() -> &'static str
    where
        Self: Sized,
    {
        "page"
    }
}
impl Default for RepositoryPage {
    fn default() -> Self {
        Self {
            page_type: PageType::None,
            content: Default::default(),
        }
    }
}

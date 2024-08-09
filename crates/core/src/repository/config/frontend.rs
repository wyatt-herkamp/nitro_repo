use badge_maker::Style;
use schemars::{schema_for, JsonSchema};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::{RepositoryConfigError, RepositoryConfigType};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default, JsonSchema)]
pub enum PageProvider {
    // Do not create a page for this projects in this repository
    #[default]
    None,
    /// The README is sent to the repository
    ReadmeSent,
}

/// Frontend Settings
#[derive(Debug, Clone, Serialize, Deserialize, Default, JsonSchema)]
#[serde(default)]
pub struct Frontend {
    pub page_provider: PageProvider,
}
#[derive(Debug, Clone, Copy, Default)]
pub struct FrontendConfigType;
impl RepositoryConfigType for FrontendConfigType {
    fn get_type(&self) -> &'static str {
        "frontend"
    }

    fn validate_config(&self, config: Value) -> Result<(), RepositoryConfigError> {
        let _config: Frontend = serde_json::from_value(config)?;
        Ok(())
    }

    fn default(&self) -> Result<Value, RepositoryConfigError> {
        Ok(serde_json::to_value(Frontend::default())?)
    }

    fn schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schema_for!(Frontend))
    }

    fn get_type_static() -> &'static str
    where
        Self: Sized,
    {
        "frontend"
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(default)]
pub struct BadgeSettings {
    pub style: BadgeStyle,
    pub label_color: String,
    pub color: String,
}
#[derive(Debug, Clone, Copy, Default)]
pub struct BadgeSettingsType;
impl RepositoryConfigType for BadgeSettingsType {
    fn get_type(&self) -> &'static str {
        "badge"
    }

    fn validate_config(&self, config: Value) -> Result<(), RepositoryConfigError> {
        let _config: BadgeSettings = serde_json::from_value(config)?;
        Ok(())
    }

    fn default(&self) -> Result<Value, RepositoryConfigError> {
        Ok(serde_json::to_value(BadgeSettings::default())?)
    }

    fn schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schema_for!(BadgeSettings))
    }

    fn get_type_static() -> &'static str
    where
        Self: Sized,
    {
        "badge"
    }
}
impl Default for BadgeSettings {
    fn default() -> Self {
        BadgeSettings {
            style: Default::default(),
            label_color: "#555".to_string(),
            color: "#33B5E5".to_string(),
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BadgeStyle(Style);
impl Default for BadgeStyle {
    fn default() -> Self {
        BadgeStyle(Style::Flat)
    }
}

impl schemars::JsonSchema for BadgeStyle {
    fn schema_name() -> std::string::String {
        "BadgeStyle".to_owned()
    }
    fn schema_id() -> std::borrow::Cow<'static, str> {
        std::borrow::Cow::Borrowed("BadgeStyle")
    }
    fn json_schema(_gen: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
        schemars::schema::Schema::Object(schemars::schema::SchemaObject {
            instance_type: Some(schemars::schema::InstanceType::String.into()),
            enum_values: Some(vec!["flat".into(), "flatsquare".into(), "plastic".into()]),
            ..Default::default()
        })
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
#[serde(tag = "page_type", content = "properties")]
pub enum RepositoryPage {
    None,
    /// Yes I am storing markdown in a json field.  I am a monster
    Markdown(String),
}
#[derive(Debug, Clone, Copy, Default)]
pub struct RepositoryPageType;
impl RepositoryConfigType for RepositoryPageType {
    fn get_type(&self) -> &'static str {
        "page"
    }

    fn validate_config(&self, config: Value) -> Result<(), RepositoryConfigError> {
        let _config: RepositoryPage = serde_json::from_value(config)?;
        Ok(())
    }

    fn default(&self) -> Result<Value, RepositoryConfigError> {
        Ok(serde_json::to_value(RepositoryPage::default())?)
    }

    fn schema(&self) -> Option<schemars::schema::RootSchema> {
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
        RepositoryPage::None
    }
}

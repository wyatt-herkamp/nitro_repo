use std::{fmt::Display, str::FromStr};

use badge_maker::Style;
use schemars::{schema_for, JsonSchema, Schema};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

use super::{RepositoryConfigError, RepositoryConfigType};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default, JsonSchema)]
pub enum PageProvider {
    /// Do not create a page for this projects in this repository
    #[default]
    NoPage,
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

    fn schema(&self) -> Option<Schema> {
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
    pub label_color: RGBColor,
    pub color: RGBColor,
}
#[derive(Debug, Clone, Copy, Default)]
pub struct BadgeSettingsType;
impl RepositoryConfigType for BadgeSettingsType {
    fn get_type(&self) -> &'static str {
        "badge"
    }
    fn get_description(&self) -> super::ConfigDescription {
        super::ConfigDescription {
            name: "Badge Settings",
            description: "Settings for the badge",
            documentation_link: None,
            ..Default::default()
        }
    }
    fn validate_config(&self, config: Value) -> Result<(), RepositoryConfigError> {
        let _config: BadgeSettings = serde_json::from_value(config)?;
        Ok(())
    }

    fn default(&self) -> Result<Value, RepositoryConfigError> {
        Ok(serde_json::to_value(BadgeSettings::default())?)
    }

    fn schema(&self) -> Option<Schema> {
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
            label_color: "#555".parse().unwrap(),
            color: "#33B5E5".parse().unwrap(),
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
    fn schema_id() -> std::borrow::Cow<'static, str> {
        std::borrow::Cow::Borrowed("BadgeStyle")
    }

    fn schema_name() -> std::borrow::Cow<'static, str> {
        std::borrow::Cow::Borrowed("BadgeStyle")
    }

    fn json_schema(_: &mut schemars::SchemaGenerator) -> schemars::Schema {
        {
            let mut map = schemars::_serde_json::Map::new();
            map.insert("type".to_owned(), "string".into());
            map.insert(
                "enum".to_owned(),
                serde_json::Value::Array({
                    let mut enum_values = Vec::new();
                    enum_values.push(("flat").into());
                    enum_values.push(("plastic").into());
                    enum_values.push(("flatquare").into());
                    enum_values
                }),
            );
            schemars::Schema::from(map)
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
#[serde(tag = "page_type", content = "properties")]
pub enum RepositoryPage {
    None,
    /// Yes I am storing markdown in a json field.  I am a monster
    #[schemars(title = "Markdown Page")]
    Markdown {
        markdown: String,
    },
}
#[derive(Debug, Clone, Copy, Default)]
pub struct RepositoryPageType;
impl RepositoryConfigType for RepositoryPageType {
    fn get_type(&self) -> &'static str {
        "page"
    }
    fn get_description(&self) -> super::ConfigDescription {
        super::ConfigDescription {
            name: "Page",
            description: "The page for the repository",
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
        RepositoryPage::None
    }
}

#[derive(Debug, Error)]
#[error("Invalid color")]
pub struct InvalidColor;
#[derive(Debug, Clone)]
pub struct RGBColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}
impl JsonSchema for RGBColor {
    fn schema_name() -> std::borrow::Cow<'static, str> {
        std::borrow::Cow::Borrowed("RGBColor")
    }

    fn json_schema(_: &mut schemars::SchemaGenerator) -> Schema {
        {
            let mut map = schemars::_serde_json::Map::new();
            map.insert("type".to_owned(), "string".into());
            map.insert("pattern".to_owned(), "^#(?:[0-9a-fA-F]{3}){1,2}$".into());
            schemars::Schema::from(map)
        }
    }
}
impl FromStr for RGBColor {
    type Err = InvalidColor;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if !s.is_ascii() {
            return Err(InvalidColor);
        }
        let s = if s.starts_with('#') {
            s[1..].to_string()
        } else {
            s.to_string()
        };

        let n = s.len();

        fn parse_single_digit(digit: &str) -> Result<u8, InvalidColor> {
            u8::from_str_radix(digit, 16)
                .map(|n| (n << 4) | n)
                .map_err(|_| InvalidColor)
        }

        if n == 3 || n == 4 {
            let r = parse_single_digit(&s[0..1])?;
            let g = parse_single_digit(&s[1..2])?;
            let b = parse_single_digit(&s[2..3])?;

            Ok(RGBColor { r, g, b })
        } else if n == 6 || n == 8 {
            let r = u8::from_str_radix(&s[0..2], 16).map_err(|_| InvalidColor)?;
            let g = u8::from_str_radix(&s[2..4], 16).map_err(|_| InvalidColor)?;
            let b = u8::from_str_radix(&s[4..6], 16).map_err(|_| InvalidColor)?;

            Ok(RGBColor { r, g, b })
        } else {
            Err(InvalidColor)
        }
    }
}
impl Display for RGBColor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "#{:02X}{:02X}{:02X}", self.r, self.g, self.b)
    }
}
impl Serialize for RGBColor {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let s = self.to_string();
        serializer.serialize_str(&s)
    }
}
impl<'de> Deserialize<'de> for RGBColor {
    fn deserialize<D>(deserializer: D) -> Result<RGBColor, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        RGBColor::from_str(&s).map_err(serde::de::Error::custom)
    }
}

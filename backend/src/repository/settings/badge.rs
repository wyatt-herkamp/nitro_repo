use badge_maker::Style;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::repository::settings::RepositoryConfigType;

#[derive(Debug, Clone, Serialize, JsonSchema, Deserialize)]
pub struct BadgeSettings {
    #[serde(default = "BadgeStyle::default")]
    pub style: BadgeStyle,
    #[serde(default = "default_label_color")]
    pub label_color: String,
    #[serde(default = "default_color")]
    pub color: String,
}

impl RepositoryConfigType for BadgeSettings {
    fn config_name() -> &'static str {
        "badge.json"
    }
}

impl Default for BadgeSettings {
    fn default() -> Self {
        BadgeSettings {
            style: Default::default(),
            label_color: default_label_color(),
            color: default_color(),
        }
    }
}

fn default_color() -> String {
    "#33B5E5".to_string()
}

fn default_label_color() -> String {
    "#555".to_string()
}

#[derive(Debug, Clone, Serialize, JsonSchema, Deserialize)]
pub enum BadgeStyle {
    Flat,
    FlatSquare,
    Plastic,
}

impl Default for BadgeStyle {
    fn default() -> Self {
        BadgeStyle::Flat
    }
}

impl BadgeStyle {
    pub fn to_badge_maker_style(&self) -> badge_maker::Style {
        match self {
            BadgeStyle::Flat => Style::Flat,
            BadgeStyle::FlatSquare => Style::FlatSquare,
            BadgeStyle::Plastic => Style::Plastic,
        }
    }
}

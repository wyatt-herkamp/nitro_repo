use badge_maker::Style;
use serde::{Deserialize, Serialize};

fn default() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Frontend {
    #[serde(default = "PageProvider::default")]
    pub page_provider: PageProvider,
    #[serde(default)]
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BadgeSettings {
    #[serde(default = "BadgeStyle::default")]
    pub style: BadgeStyle,
    #[serde(default = "default_label_color")]
    pub label_color: String,
    #[serde(default = "default_color")]
    pub color: String,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PageProvider {
    None,
    README,
    ReadmeGit,
    ReadmeSent,
}

impl PageProvider {
    fn default() -> Self {
        PageProvider::None
    }
}

impl Default for Frontend {
    fn default() -> Self {
        Frontend {
            page_provider: PageProvider::None,
            description: "".to_string()
        }
    }
}

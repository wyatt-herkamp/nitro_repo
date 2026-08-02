pub(crate) mod parser;
pub(crate) mod util;

use std::fmt::{Display, Formatter};

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum Color {
    Named(NamedColor),
    Alias(AliasColor),
    Rgb(u8, u8, u8),
}

impl Display for Color {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Color::Named(named) => {
                write!(f, "{}", named.hex())
            }

            Color::Alias(alias) => {
                write!(f, "{}", alias.hex())
            }
            Color::Rgb(r, g, b) => {
                write!(f, "rgb({},{},{})", r, g, b)
            }
        }
    }
}

/// Colors from the
/// [badge-maker](https://github.com/badges/shields/blob/master/badge-maker/lib/color.js) spec
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum NamedColor {
    BrightGreen,
    Green,
    Yellow,
    YellowGreen,
    Orange,
    Red,
    Blue,
    Grey,
    LightGrey,
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum AliasColor {
    Gray,
    LightGray,
    Critical,
    Important,
    Success,
    Informational,
    Inactive,
}

impl NamedColor {
    pub fn hex(&self) -> &'static str {
        match self {
            NamedColor::BrightGreen => "#4c1",
            NamedColor::Green => "#97ca00",
            NamedColor::Yellow => "#dfb317",
            NamedColor::YellowGreen => "#a4a61d",
            NamedColor::Orange => "#fe7d37",
            NamedColor::Red => "#e05d44",
            NamedColor::Blue => "#007ec6",
            NamedColor::Grey => "#555",
            NamedColor::LightGrey => "#9f9f9f",
        }
    }
}

impl AliasColor {
    pub fn hex(&self) -> &'static str {
        match self {
            AliasColor::Gray => NamedColor::Grey.hex(),
            AliasColor::LightGray => NamedColor::LightGrey.hex(),
            AliasColor::Critical => NamedColor::Red.hex(),
            AliasColor::Important => NamedColor::Orange.hex(),
            AliasColor::Success => NamedColor::BrightGreen.hex(),
            AliasColor::Informational => NamedColor::Blue.hex(),
            AliasColor::Inactive => NamedColor::LightGrey.hex(),
        }
    }
}

impl From<AliasColor> for Color {
    fn from(alias: AliasColor) -> Self {
        Color::Alias(alias)
    }
}

impl From<NamedColor> for Color {
    fn from(named: NamedColor) -> Self {
        Color::Named(named)
    }
}

impl From<(u8, u8, u8)> for Color {
    fn from((r, g, b): (u8, u8, u8)) -> Self {
        Self::Rgb(r, g, b)
    }
}

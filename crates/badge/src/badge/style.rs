use std::{
    convert::TryFrom,
    fmt::{Display, Formatter},
    str::FromStr,
};

use serde::Serialize;

use crate::{
    badge::style::Style::*,
    error::{Error, Error::BadStyleChoice},
};

/// Used to define the style of a badge. Used in [BadgeBuilder.style()](crate::badge::BadgeBuilder)
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum Style {
    Flat,
    Plastic,
    FlatSquare,
    // ForTheBadge,
    // Social,
}
impl FromStr for Style {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "flat" => Ok(Flat),
            "plastic" => Ok(Plastic),
            "flatsquare" => Ok(FlatSquare),
            // "forthebadge" => Ok(ForTheBadge),
            // "social" => Ok(Social),
            choice => Err(BadStyleChoice(choice.to_string())),
        }
    }
}
impl TryFrom<String> for Style {
    type Error = Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        value.parse()
    }
}
impl TryFrom<&str> for Style {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        value.parse()
    }
}
impl Serialize for Style {
    fn serialize<S>(
        &self,
        serializer: S,
    ) -> Result<<S as serde::Serializer>::Ok, <S as serde::Serializer>::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.to_string().as_str())
    }
}
impl<'de> serde::Deserialize<'de> for Style {
    fn deserialize<D>(deserializer: D) -> Result<Style, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Style::try_from(s).map_err(serde::de::Error::custom)
    }
}

impl Display for Style {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Flat => "flat",
                Plastic => "plastic",
                FlatSquare => "flatsquare",
                // ForTheBadge => "forthebadge",
                // Social => "social",
            }
        )
    }
}

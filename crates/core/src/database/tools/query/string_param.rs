use std::fmt::Display;

use serde::{
    Serialize,
    de::{Deserialize, Deserializer, Visitor},
};
use sqlx::query_builder::Separated;
use tracing::instrument;
use utoipa::ToSchema;

use crate::database::tools::ColumnType;

/// String Param Lookup that allows exact and like queries
#[derive(Debug, Clone, PartialEq, Eq, Serialize, ToSchema)]
#[serde(tag = "type", content = "value")]
pub enum StringParamType {
    /// Exact query. Case is ignored. Will be converted to lowercase
    Exact(String),
    /// Like query Will append % to the front and back of the string
    Like(String),
    /// Exists for when the value is empty. Because having the variants take an Option<String> is annoying.
    None,
}
impl StringParamType {
    pub fn is_none(&self) -> bool {
        matches!(self, Self::None)
    }

    #[instrument(name = "StringParamType::push", skip(separated))]
    pub fn push<C>(&self, column_name: C, separated: &mut Separated<'_, '_, sqlx::Postgres, &str>)
    where
        C: ColumnType + Display,
    {
        match self {
            StringParamType::Exact(value) => {
                separated.push(format!("LOWER({}) = ", column_name));
                separated.push_bind_unseparated(value.to_lowercase());
            }
            // This is a like query
            StringParamType::Like(value) => {
                separated.push(format!("{} LIKE ", column_name));
                separated.push_bind_unseparated(format!("%{}%", value));
            }
            // No need to push anything
            StringParamType::None => {}
        }
    }
}
impl Default for StringParamType {
    fn default() -> Self {
        Self::None
    }
}
enum Variant {
    Exact,
    Like,
}
struct StringParamTypeVisitor;
impl<'de> Visitor<'de> for StringParamTypeVisitor {
    type Value = StringParamType;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a string param type")
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        let value = value.trim();
        if value.is_empty() {
            return Ok(StringParamType::None);
        }
        Ok(StringParamType::Like(value.to_owned()))
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
        let mut variant = None;
        let mut value = None;

        while let Some((key, v)) = map.next_entry::<String, String>()? {
            match key.as_str() {
                "type" => match v.as_str() {
                    "Exact" => variant = Some(Variant::Exact),
                    "Like" => variant = Some(Variant::Like),
                    _ => return Err(serde::de::Error::unknown_variant(&v, &["Exact", "Like"])),
                },
                "value" => {
                    value = Some(v);
                }
                _ => {
                    return Err(serde::de::Error::unknown_field(&key, &["type", "value"]));
                }
            }
        }

        let variant = variant.ok_or_else(|| serde::de::Error::missing_field("type"))?;
        let value = value
            .ok_or_else(|| serde::de::Error::missing_field("value"))?
            .trim()
            .to_owned();
        if value.is_empty() {
            return Ok(StringParamType::None);
        }
        match variant {
            Variant::Exact => Ok(StringParamType::Exact(value)),
            Variant::Like => Ok(StringParamType::Like(value)),
        }
    }
}
impl<'de> Deserialize<'de> for StringParamType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(StringParamTypeVisitor)
    }
}

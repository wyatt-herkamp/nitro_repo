use std::str::FromStr;

use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgValueRef, Decode, Encode};
use strum::{AsRefStr, Display, EnumIs, EnumString, IntoStaticStr};
use utoipa::ToSchema;
/// Release type of a project
///
/// Can be overridden in the panel.
#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    Hash,
    Serialize,
    Deserialize,
    ToSchema,
    EnumIs,
    IntoStaticStr,
    Display,
    EnumString,
)]
pub enum ReleaseType {
    /// Stable Release
    Stable,
    /// Beta Release
    Beta,
    /// Alpha Release
    Alpha,
    /// Snapshot Release
    /// Only really used in Maven
    Snapshot,
    /// .RC Release
    ReleaseCandidate,
    /// The release type could not be determined
    Unknown,
}
impl<'q, DB: ::sqlx::Database> Encode<'q, DB> for ReleaseType
where
    &'q str: Encode<'q, DB>,
{
    fn encode_by_ref(
        &self,
        buf: &mut <DB as ::sqlx::database::Database>::ArgumentBuffer<'q>,
    ) -> ::std::result::Result<::sqlx::encode::IsNull, ::sqlx::error::BoxDynError> {
        let val: &str = self.into();
        <&str as Encode<'q, DB>>::encode(val, buf)
    }
    fn size_hint(&self) -> ::std::primitive::usize {
        let val = self.into();
        <&str as Encode<'q, DB>>::size_hint(&val)
    }
}
#[automatically_derived]
impl<'r> Decode<'r, ::sqlx::postgres::Postgres> for ReleaseType {
    fn decode(
        value: PgValueRef<'r>,
    ) -> Result<Self, Box<dyn std::error::Error + 'static + Send + Sync>> {
        let value = <&'r str as Decode<'r, ::sqlx::postgres::Postgres>>::decode(value)?;
        ReleaseType::from_str(value)
            .map_err(|_| format!("invalid value {:?} for enum {}", value, "ReleaseType").into())
    }
}
#[automatically_derived]
impl ::sqlx::Type<::sqlx::Postgres> for ReleaseType {
    fn type_info() -> ::sqlx::postgres::PgTypeInfo {
        ::sqlx::postgres::PgTypeInfo::with_name("TEXT")
    }
}
#[automatically_derived]
impl ::sqlx::postgres::PgHasArrayType for ReleaseType {
    fn array_type_info() -> ::sqlx::postgres::PgTypeInfo {
        ::sqlx::postgres::PgTypeInfo::array_of("TEXT")
    }
}
impl Default for ReleaseType {
    fn default() -> Self {
        ReleaseType::Unknown
    }
}
impl ReleaseType {
    pub fn release_type_from_version(version: &str) -> ReleaseType {
        let version = version.to_lowercase();
        if version.contains("snapshot") {
            ReleaseType::Snapshot
        } else if version.contains("beta") {
            ReleaseType::Beta
        } else if version.contains("alpha") {
            ReleaseType::Alpha
        } else if version.contains(".rc") {
            ReleaseType::ReleaseCandidate
        } else {
            ReleaseType::Stable
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema, sqlx::Type)]
pub enum ProjectState {
    Active,
    Deprecated,
}
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema, Default, Builder)]
#[serde(default)]
pub struct VersionData {
    #[builder(default)]
    pub documentation_url: Option<String>,
    #[builder(default)]
    pub website: Option<String>,
    #[serde(default)]
    #[builder(default)]
    pub authors: Vec<Author>,
    #[builder(default)]
    pub description: Option<String>,
    #[builder(default)]
    pub source: Option<ProjectSource>,
    #[builder(default)]
    pub licence: Option<Licence>,
}
/// Author of the project
///
/// All data is optional as artifact types may not have all the data
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
pub struct Author {
    /// Name of the author
    pub name: Option<String>,
    pub email: Option<String>,
    pub website: Option<String>,
}
/// Source of the project
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
#[serde(tag = "type", content = "value")]
pub enum ProjectSource {
    /// A git repository
    Git { url: String },
}
/// Licence of the project Two Different types depending on how the artifact is setup
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
#[serde(tag = "type", content = "value")]
pub enum Licence {
    Simple(String),
    Array(Vec<LicenceValue>),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
pub struct LicenceValue {
    /// Licence Name
    pub name: String,
    /// Licence URL
    pub url: Option<String>,
}

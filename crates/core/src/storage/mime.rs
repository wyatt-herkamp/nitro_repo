use derive_more::derive::{Deref, From, Into};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use utoipa::ToSchema;

#[derive(Debug, Clone, From, Into, Deref, ToSchema)]
#[schema(value_type = String)]
pub struct SerdeMime(pub mime::Mime);

impl Serialize for SerdeMime {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.0.to_string().serialize(serializer)
    }
}
impl<'de> Deserialize<'de> for SerdeMime {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let string = String::deserialize(deserializer)?;
        Ok(SerdeMime(string.parse().map_err(serde::de::Error::custom)?))
    }
}

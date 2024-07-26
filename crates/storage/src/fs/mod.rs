mod content;
mod file;
mod file_meta;
mod path;
pub(crate) mod utils;

pub use content::*;
use derive_more::{From, Into};
pub use file::*;
pub use file_meta::*;
pub use path::StoragePath;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
#[derive(Debug, Clone, From, Into)]
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

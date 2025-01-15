use ahash::HashMap;
use serde::Serialize;
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RepositoryMeta(pub HashMap<String, String>);

impl Serialize for RepositoryMeta {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.0.serialize(serializer)
    }
}

impl<'de> serde::Deserialize<'de> for RepositoryMeta {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Ok(RepositoryMeta(HashMap::deserialize(deserializer)?))
    }
}

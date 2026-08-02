use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct CharDatum {
    pub low: u32,
    pub high: u32,
    pub width: f32,
}

impl CharDatum {
    pub(crate) fn contains(&self, char: u32) -> bool {
        char >= self.low && char <= self.high
    }
}

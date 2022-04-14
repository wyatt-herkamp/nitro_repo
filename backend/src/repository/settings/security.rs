use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, strum_macros::EnumString)]
pub enum Visibility {
    Public,
    Private,
    Hidden,
}

impl Default for Visibility {
    fn default() -> Self {
        Visibility::Public
    }
}


#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct SecurityRules {
    #[serde(default = "Visibility::default")]
    pub visibility: Visibility,

}

impl SecurityRules {
    pub fn update(&mut self, security: SecurityRules) {
        self.visibility = security.visibility;
    }
    pub fn set_visibility(&mut self, visibility: Visibility) {
        self.visibility = visibility;
    }
}

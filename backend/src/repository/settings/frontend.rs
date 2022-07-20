use serde::{Deserialize, Serialize};

use crate::repository::settings::RepositoryConfigType;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PageProvider {
    // Do not create a page for this projects in this repository
    None,
    /// The README is pulled from Github
    ReadmeGit,
    /// The README is sent to the repository
    ReadmeSent,
}

impl Default for PageProvider {
    fn default() -> Self {
        PageProvider::None
    }
}

/// Frontend Settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Frontend {
    pub page_provider: PageProvider,
    /// The Description of the Repository
    #[serde(default)]
    pub description: String,
}

impl RepositoryConfigType for Frontend {
    fn config_name() -> &'static str {
        "frontend.json"
    }
}

impl Default for Frontend {
    fn default() -> Self {
        Frontend {
            page_provider: PageProvider::None,
            description: "".to_string(),
        }
    }
}

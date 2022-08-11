use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::repository::settings::RepositoryConfigType;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default, JsonSchema)]
pub enum PageProvider {
    // Do not create a page for this projects in this repository
    #[default]
    None,
    /// The README is pulled from Github
    ReadmeGit,
    /// The README is sent to the repository
    ReadmeSent,
}

/// Frontend Settings
#[derive(Debug, Clone, Serialize, Deserialize, Default, JsonSchema)]
pub struct Frontend {
    #[serde(default)]
    pub page_provider: PageProvider,
}

impl RepositoryConfigType for Frontend {

    fn config_name() -> &'static str {
        "frontend.json"
    }
}
pub mod multi_web {
    use crate::repository;
    use crate::repository::settings;
    repository::web::multi::settings::define_repository_config_handlers_group!(
        settings::frontend::Frontend,
        frontend,
        Maven
    );
}

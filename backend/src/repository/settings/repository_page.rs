use crate::repository::settings::RepositoryConfigType;
use crate::repository::web::multi::configs::define_repository_config_handlers;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub enum PageType {
    #[default]
    None,
    Markdown(String),
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RepositoryPage {
    #[serde(default)]
    pub page_type: PageType,
}

impl RepositoryConfigType for RepositoryPage {
    fn config_name() -> &'static str {
        "repository_page.json"
    }
}

define_repository_config_handlers!("repository_page", RepositoryPage);

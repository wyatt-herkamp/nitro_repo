use crate::repository::settings::RepositoryConfigType;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq, Eq, JsonSchema)]
pub enum PageType {
    #[default]
    None,
    Markdown,
}

#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
pub struct RepositoryPage {
    #[serde(default)]
    pub page_type: PageType,
}

impl RepositoryConfigType for RepositoryPage {
    fn config_name() -> &'static str {
        "page.json"
    }
}

pub mod multi_web {
    use crate::repository;
    use crate::repository::settings;
    //repository::web::multi::settings::define_repository_config_handlers_group!(
    //    settings::repository_page::RepositoryPage,
    //    page,
    //     Maven
    //);
}

use std::fs::create_dir_all;
use crate::settings::models::{EmailSetting, SecuritySettings, SiteSetting};
use crate::Settings;

use std::path::{Path, PathBuf};
use tokio::fs::read_to_string;

pub mod models;


pub async fn load_configs(configs: PathBuf) -> anyhow::Result<Settings> {
    let security: SecuritySettings =
        toml::from_str(&read_to_string(configs.join("security.toml")).await?)?;
    let site: SiteSetting = toml::from_str(&read_to_string(configs.join("site.toml")).await?)?;
    let email: EmailSetting = toml::from_str(&read_to_string(configs.join("email.toml")).await?)?;

    Ok(Settings {
        email,
        site,
        security,
    })
}

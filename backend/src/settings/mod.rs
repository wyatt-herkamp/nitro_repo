use crate::settings::models::{EmailSetting, SecuritySettings, SiteSetting};
use crate::Settings;
use actix_web::web;
use std::path::Path;
use tokio::fs::read_to_string;

pub mod models;

pub async fn load_configs() -> anyhow::Result<Settings> {
    let cfgs = Path::new("cfg");

    let security: SecuritySettings =
        toml::from_str(&read_to_string(cfgs.join("security.toml")).await?)?;
    let site: SiteSetting = toml::from_str(&read_to_string(cfgs.join("site.toml")).await?)?;
    let email: EmailSetting = toml::from_str(&read_to_string(cfgs.join("email.toml")).await?)?;

    Ok(Settings {
        email,
        site,
        security,
    })
}

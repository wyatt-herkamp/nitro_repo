use std::path::Path;
use actix_web::web;
use tokio::fs::{read_to_string};
use crate::Settings;
use crate::settings::models::{EmailSetting, SecuritySettings, SiteSetting};

pub mod controller;
pub mod models;

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(controller::setting_report);
}

pub async fn load_configs() -> anyhow::Result<Settings> {
    let cfgs = Path::new("cfg");

    let security: SecuritySettings = toml::from_str(&read_to_string(cfgs.join("security.toml")).await?)?;
    let site: SiteSetting = toml::from_str(&read_to_string(cfgs.join("site.toml")).await?)?;
    let email: EmailSetting = toml::from_str(&read_to_string(cfgs.join("email.toml")).await?)?;

    Ok(Settings {
        email,
        site,
        security,
    })
}

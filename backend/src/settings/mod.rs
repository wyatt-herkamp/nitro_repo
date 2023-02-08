use std::io;
use std::path::PathBuf;

use tokio::fs::read_to_string;

use crate::settings::models::{EmailSetting, SecuritySettings, SiteSetting};
use crate::Settings;

pub mod models;

pub async fn load_configs(configs: PathBuf) -> io::Result<Settings> {
    let security: SecuritySettings =
        toml::from_str(&read_to_string(configs.join("security.toml")).await?)
            .map_err(toml_to_io_error)?;
    let site: SiteSetting = toml::from_str(&read_to_string(configs.join("site.toml")).await?)
        .map_err(toml_to_io_error)?;
    let email: EmailSetting = toml::from_str(&read_to_string(configs.join("email.toml")).await?)
        .map_err(toml_to_io_error)?;

    Ok(Settings {
        email,
        site,
        security,
    })
}
fn toml_to_io_error(err: toml::de::Error) -> io::Error {
    io::Error::new(io::ErrorKind::Other, err)
}

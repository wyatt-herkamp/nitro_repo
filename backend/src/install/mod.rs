use crate::settings::models::{EmailSetting, SecuritySettings, SiteSetting};
use crate::GeneralSettings;
use log::info;
use std::fs::{create_dir_all, OpenOptions};
use std::io;
use std::io::Write;
use std::path::PathBuf;

pub fn install_data(working_dir: PathBuf, general: GeneralSettings) -> io::Result<()> {
    let configs = working_dir.join("cfg");
    create_dir_all(&configs)?;

    let other = toml::to_string_pretty(&general).unwrap();
    let mut file = OpenOptions::new()
        .write(true)
        .append(true)
        .create(true)
        .open(working_dir.join("nitro_repo.toml"))?;
    file.write_all(other.as_bytes())?;

    let security = toml::to_string_pretty(&SecuritySettings::default()).unwrap();
    let mut file = OpenOptions::new()
        .write(true)
        .append(true)
        .create(true)
        .open(configs.join("security.toml"))?;
    file.write_all(security.as_bytes())?;

    let email = toml::to_string_pretty(&EmailSetting::default()).unwrap();
    let mut file = OpenOptions::new()
        .write(true)
        .append(true)
        .create(true)
        .open(configs.join("email.toml"))?;
    file.write_all(email.as_bytes())?;
    let site = toml::to_string_pretty(&SiteSetting::default()).unwrap();
    let mut file = OpenOptions::new()
        .write(true)
        .append(true)
        .create(true)
        .open(configs.join("site.toml"))?;
    file.write_all(site.as_bytes())?;
    info!("Installation Complete");
    Ok(())
}

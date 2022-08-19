use crate::configs::user::UserConfig;
use std::fs::{read_to_string, OpenOptions};
use std::io::Write;


pub mod user;

pub fn get_user_config() -> anyhow::Result<UserConfig> {
    let dir = directories::UserDirs::new()
        .unwrap()
        .home_dir()
        .join(".nrc");
    if !dir.exists() {
        std::fs::create_dir_all(&dir)?;
    }
    let buf = dir.join("config.toml");
    if !buf.exists() {
        let config = UserConfig::default();
        let content = toml::to_string_pretty(&config)?;
        OpenOptions::new()
            .write(true)
            .create(true)
            .open(&buf)?
            .write_all(content.as_bytes())?;
        Ok(config)
    } else {
        let string = read_to_string(&buf)?;
        let config = toml::from_str::<UserConfig>(&string)?;
        Ok(config)
    }
}

pub fn save_user_config(config: &UserConfig) -> anyhow::Result<()> {
    let file = directories::UserDirs::new()
        .unwrap()
        .home_dir()
        .join(".nrc")
        .join("config.toml");
    let mut file = OpenOptions::new().write(true).append(false).open(&file)?;
    let content = toml::to_string_pretty(config)?;

    file.write_all(content.as_bytes())?;
    Ok(())
}

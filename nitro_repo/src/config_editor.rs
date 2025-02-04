use std::path::PathBuf;

use inquire::{Text, validator::Validation};
use nr_core::database::DatabaseConfig;
use sqlx::{Connection, PgConnection, postgres::PgConnectOptions};

use crate::app::config::ReadConfigType;

#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum ConfigSection {
    Database,
}

pub async fn editor(section: ConfigSection, config_path: PathBuf) -> anyhow::Result<()> {
    let mut config = if config_path.exists() {
        let config = std::fs::read_to_string(&config_path)?;
        toml::from_str(&config)?
    } else {
        ReadConfigType::default()
    };

    match section {
        ConfigSection::Database => {
            let new_database = edit_database(config.database.unwrap_or_default()).await?;
            config.database = Some(new_database);
        }
    }

    let new_config = toml::to_string_pretty(&config)?;

    std::fs::write(&config_path, new_config)?;
    Ok(())
}

async fn edit_database(database: DatabaseConfig) -> anyhow::Result<DatabaseConfig> {
    let port_validator = |port: &str| {
        if let Ok(number) = port.parse::<u16>() {
            if number > 0 && number < 65535 {
                Ok(Validation::Valid)
            } else {
                Ok(Validation::Invalid(
                    "Port must be between 1 and 65535".into(),
                ))
            }
        } else {
            Ok(Validation::Invalid("Port must be a number".into()))
        }
    };
    let old_database_port = database
        .port
        .map(|p| p.to_string())
        .unwrap_or_else(|| "5432".to_string());
    let host = Text::new("Database Host")
        .with_default(&database.host)
        .prompt()?;
    let port = Text::new("Database Port")
        .with_default(&old_database_port)
        .with_validator(port_validator)
        .prompt()?;
    let port_as_number = port.parse::<u16>()?;

    let user = Text::new("Database User")
        .with_default(&database.user)
        .prompt()?;

    let password = Text::new("Database Password")
        .with_default(&database.password)
        .prompt()?;

    let database_name = Text::new("Database Name")
        .with_default(&database.database)
        .prompt()?;

    let database = DatabaseConfig {
        host,
        port: Some(port_as_number),
        user,
        password,
        database: database_name,
    };
    {
        let options: PgConnectOptions = database.clone().try_into()?;
        let mut conn = PgConnection::connect_with(&options).await?;
        conn.ping().await?;
        conn.close().await?;
    }
    Ok(database)
}

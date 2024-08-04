use config_types::size_config::{ConfigSize, Unit};
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgConnectOptions;
use std::env;
use std::path::PathBuf;

use super::authentication::session::SessionManagerConfig;
use super::email::EmailSetting;
use super::logging::LoggingConfig;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum Mode {
    Debug,
    Release,
}
impl Default for Mode {
    fn default() -> Self {
        #[cfg(debug_assertions)]
        return Mode::Debug;
        #[cfg(not(debug_assertions))]
        return Mode::Release;
    }
}
pub fn get_current_directory() -> PathBuf {
    env::current_dir().unwrap_or_else(|_| PathBuf::new())
}
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SecuritySettings {
    pub allow_basic_without_tokens: bool,
}
impl Default for SecuritySettings {
    fn default() -> Self {
        Self {
            allow_basic_without_tokens: false,
        }
    }
}
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct NitroRepoConfig {
    pub database: PostgresSettings,
    pub log: LoggingConfig,
    pub bind_address: String,
    pub max_upload: ConfigSize,
    pub server_workers: Option<usize>,
    pub mode: Mode,
    pub sessions: SessionManagerConfig,
    pub tls: Option<TlsConfig>,
    pub email: EmailSetting,
    pub site: SiteSetting,
    pub security: SecuritySettings,
}
impl NitroRepoConfig {
    pub fn load(config_file: PathBuf, update_config: bool) -> anyhow::Result<Self> {
        let app = if config_file.exists() {
            let config = std::fs::read_to_string(&config_file)?;
            let app: NitroRepoConfig = toml::from_str(&config)?;
            if update_config {
                let toml = toml::to_string_pretty(&app)?;
                std::fs::write(&config_file, &toml)?;
            }
            app
        } else {
            let default = NitroRepoConfig::default();
            let config = toml::to_string_pretty(&default)?;
            std::fs::write(&config_file, &config)?;
            default
        };
        Ok(app)
    }
}

impl Default for NitroRepoConfig {
    fn default() -> Self {
        Self {
            database: PostgresSettings::default(),
            log: LoggingConfig::default(),
            bind_address: "0.0.0.0:6742".to_string(),
            max_upload: ConfigSize {
                size: 1024,
                unit: Unit::Mebibytes,
            },
            server_workers: None,
            mode: Mode::default(),
            tls: None,
            sessions: SessionManagerConfig::default(),
            email: EmailSetting::default(),
            site: SiteSetting::default(),
            security: SecuritySettings::default(),
        }
    }
}
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PostgresSettings {
    pub user: String,
    pub password: String,
    pub host: String,
    pub database: String,
}
impl Default for PostgresSettings {
    fn default() -> Self {
        Self {
            user: "postgres".to_string(),
            password: "password".to_string(),
            host: "localhost:5432".to_string(),
            database: "nitro_repo".to_string(),
        }
    }
}
impl From<PostgresSettings> for PgConnectOptions {
    fn from(settings: PostgresSettings) -> Self {
        let host = settings.host.split(':').collect::<Vec<&str>>();
        let (host, port) = match host.len() {
            1 => (host[0], 5432),
            2 => (host[0], host[1].parse::<u16>().unwrap_or(5432)),
            _ => ("localhost", 5432),
        };
        PgConnectOptions::new()
            .username(&settings.user)
            .password(&settings.password)
            .host(host)
            .port(port)
            .database(&settings.database)
    }
}
#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct TlsConfig {
    pub private_key: PathBuf,
    pub certificate_chain: PathBuf,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(default)]
pub struct SiteSetting {
    /// If not set, the app will load the url from the request.
    pub app_url: Option<String>,
    pub name: String,
    pub description: String,
    pub is_https: bool,
}

impl Default for SiteSetting {
    fn default() -> Self {
        SiteSetting {
            app_url: None,
            name: "Nitro Repo".to_string(),
            description: "An Open Source artifact manager.".to_string(),
            is_https: false,
        }
    }
}

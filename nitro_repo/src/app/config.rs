use nr_core::database::DatabaseConfig;
use serde::{Deserialize, Serialize};
use std::env;
use std::fs::read_to_string;
use std::path::PathBuf;
use strum::EnumIs;
use tuxs_config_types::size_config::InvalidSizeError;
use utoipa::ToSchema;
mod max_upload;
mod security;
use super::authentication::session::SessionManagerConfig;
use super::email::EmailSetting;
use crate::logging::config::LoggingConfig;
use crate::repository::StagingConfig;
pub use max_upload::*;
pub use security::*;
pub const CONFIG_PREFIX: &str = "NITRO-REPO";
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Invalid size: {0}")]
    InvalidSize(#[from] InvalidSizeError),
    #[error(
        "Invalid max upload size. Expected a valid size or 'unlimited', error: {error}, got: {value}"
    )]
    InvalidMaxUpload {
        error: InvalidSizeError,
        value: String,
    },
}
#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq, Eq, EnumIs, ToSchema)]
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

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
#[serde(default)]
pub struct NitroRepoConfig {
    pub mode: Mode,
    pub web_server: WebServer,
    pub suggested_local_storage_path: Option<PathBuf>,
    pub database: DatabaseConfig,
    pub log: LoggingConfig,
    pub sessions: SessionManagerConfig,
    pub site: SiteSetting,
    pub security: SecuritySettings,
    pub staging: StagingConfig,
    pub email: Option<EmailSetting>,
}
#[derive(Debug, Deserialize, Serialize, Clone, Default)]
#[serde(default)]
pub struct ReadConfigType {
    pub mode: Option<Mode>,
    pub suggested_local_storage_path: Option<PathBuf>,
    pub web_server: Option<WebServer>,
    pub database: Option<DatabaseConfig>,
    pub log: Option<LoggingConfig>,
    pub sessions: Option<SessionManagerConfig>,
    pub email: Option<EmailSetting>,
    pub site: Option<SiteSetting>,
    pub security: Option<SecuritySettings>,
    pub staging: Option<StagingConfig>,
}
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct WebServer {
    pub bind_address: String,
    /// Should OpenAPI routes be enabled.
    pub open_api_routes: bool,
    /// The maximum upload size for the web server.
    pub max_upload: MaxUpload,
    /// The TLS configuration for the web server.
    pub tls: Option<TlsConfig>,
}
impl Default for WebServer {
    fn default() -> Self {
        Self {
            bind_address: "0.0.0.0:6742".to_owned(),
            open_api_routes: true,
            max_upload: Default::default(),
            tls: None,
        }
    }
}
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(default)]
pub struct SiteSetting {
    /// If not set, the app will load the url from the request.
    pub app_url: Option<String>,
    pub name: String,
    pub description: String,
    pub is_https: bool,
    #[cfg(feature = "frontend")]
    pub frontend_path: Option<PathBuf>,
}

impl Default for SiteSetting {
    fn default() -> Self {
        SiteSetting {
            app_url: None,
            name: "Nitro Repo".to_string(),
            description: "An Open Source artifact manager.".to_string(),
            is_https: false,
            #[cfg(feature = "frontend")]
            frontend_path: None,
        }
    }
}

macro_rules! env_or_file_or_default {
    (
        $config:ident,
        $env:ident,
        $key:ident
    ) => {
        $config.$key.or($env.$key).unwrap_or_default()
    };
    ( $config:ident, $env:ident, $($key:ident),* ) => {
        (
            $(
                env_or_file_or_default!($config, $env, $key),
            )*
        )
    }
}
macro_rules! env_or_file_or_none {
    (
        $config:ident,
        $env:ident,
        $key:ident
    ) => {
        $config.$key.or($env.$key)
    };
    ( $config:ident, $env:ident, $($key:ident),* ) => {
        (
            $(
                env_or_file_or_none!($config, $env, $key),
            )*
        )
    }
}
/// Load the configuration from the environment or a configuration file.
///
/// path: may not exist if it doesn't it will use the environment variables.
///
/// Config File gets precedence over environment variables.
pub fn load_config(path: Option<PathBuf>) -> anyhow::Result<NitroRepoConfig> {
    let environment: ReadConfigType = serde_env::from_env_with_prefix(CONFIG_PREFIX)?;
    let config_from_file = if let Some(path) = path.filter(|path| path.exists() && path.is_file()) {
        let contents = read_to_string(path)?;
        toml::from_str(&contents)?
    } else {
        ReadConfigType::default()
    };
    // Merge the environment variables with the configuration file. If neither exists the default values are used.
    // Environment variables take precedence.
    let (mode, web_server, database, log, sessions, site, security, staging) = env_or_file_or_default!(
        config_from_file,
        environment,
        mode,
        web_server,
        database,
        log,
        sessions,
        site,
        security,
        staging
    );
    let email = env_or_file_or_none!(config_from_file, environment, email);
    let suggested_local_storage_path =
        env_or_file_or_none!(config_from_file, environment, suggested_local_storage_path);
    Ok(NitroRepoConfig {
        mode,
        web_server,
        database,
        log,
        sessions,
        site,
        security,
        staging,
        email,
        suggested_local_storage_path,
    })
}

use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::path::PathBuf;

use sea_orm::ConnectOptions;
use semver::{Error, Version};
use serde::{Deserialize, Serialize};
use toml::Value;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum Mode {
    Debug,
    Release,
    Install,
}

impl AsRef<Mode> for Mode {
    fn as_ref(&self) -> &Mode {
        self
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Settings {
    pub email: EmailSetting,
    pub site: SiteSetting,
    pub security: SecuritySettings,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Internal {
    pub installed: bool,
    pub version: String,
}

impl Internal {
    pub fn parse_version(&self) -> Result<Version, Error> {
        semver::Version::parse(&self.version)
    }
}

impl Default for Internal {
    fn default() -> Self {
        Self {
            installed: true,
            version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(tag = "type", content = "settings")]
pub enum Database {
    Mysql(MysqlSettings),
}
#[allow(clippy::from_over_into)]
impl Into<sea_orm::ConnectOptions> for Database {
    fn into(self) -> ConnectOptions {
        match self {
            Database::Mysql(mysql) => ConnectOptions::new(mysql.to_string()),
        }
    }
}

pub type StringMap = HashMap<String, String>;

impl Display for MysqlSettings {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "mysql://{}:{}@{}/{}",
            self.user, self.password, self.host, self.database
        )
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct MysqlSettings {
    pub user: String,
    pub password: String,
    pub host: String,
    pub database: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Application {
    pub log: String,
    pub address: String,
    pub app_url: String,
    pub max_upload: usize,
    pub mode: Mode,
    pub storage_location: PathBuf,
    pub ssl_private_key: Option<String>,
    pub ssl_cert_key: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct GeneralSettings {
    pub database: Database,
    pub application: Application,
    pub internal: Internal,
    #[serde(default)]
    pub session: SessionSettings,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SiteSetting {
    pub name: String,
    pub description: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SessionSettings {
    pub manager: String,
    pub value: Option<Value>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SecuritySettings {}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct EmailSetting {
    pub username: String,
    pub password: String,
    pub host: String,
    pub encryption: String,
    pub from: String,
    pub port: u16,
}

#[allow(clippy::derivable_impls)]
impl Default for SecuritySettings {
    fn default() -> Self {
        SecuritySettings {}
    }
}

impl Default for SiteSetting {
    fn default() -> Self {
        SiteSetting {
            name: "nitro_repo".to_string(),
            description: "nitro_repo".to_string(),
        }
    }
}

impl Default for EmailSetting {
    fn default() -> Self {
        EmailSetting {
            username: "no-reply@example.com".to_string(),
            password: "".to_string(),
            host: "example.com".to_string(),
            encryption: "TLS".to_string(),
            from: "no-reply@example.com".to_string(),
            port: 587,
        }
    }
}

impl Default for SessionSettings {
    fn default() -> Self {
        SessionSettings {
            manager: "BasicSessionManager".to_string(),
            value: None,
        }
    }
}

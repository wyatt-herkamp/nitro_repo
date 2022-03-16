use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use serde::{Deserialize, Serialize};
use crate::error::internal_error::InternalError;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum Mode {
    Debug,
    Release,
    Install,
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

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Database<T> {
    #[serde(rename = "type")]
    pub db_type: String,
    #[serde(flatten)]
    pub settings: T,
}

pub type StringMap = HashMap<String, String>;
pub type GenericDatabase = Database<StringMap>;


impl Display for MysqlSettings {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "mysql://{}:{}@{}/{}",
            self.user,
            self.password,
            self.host,
            self.database
        )
    }
}

impl TryFrom<StringMap> for MysqlSettings {
    type Error = InternalError;

    fn try_from(mut value: StringMap) -> Result<Self, Self::Error> {
        let user = value.remove("user").ok_or_else(|| InternalError::ConfigError("database.user".to_string()))?;
        let password = value.remove("password").ok_or_else(|| InternalError::ConfigError("database.password".to_string()))?;
        let host = value.remove("password").ok_or_else(|| InternalError::ConfigError("database.host".to_string()))?;
        let database = value.remove("password").ok_or_else(|| InternalError::ConfigError("database.database".to_string()))?;
        return Ok(MysqlSettings {
            user,
            password,
            host,
            database,
        });
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
    pub frontend: String,
    pub log: String,
    pub address: String,
    pub max_upload: usize,
    pub mode: Mode,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct GeneralSettings {
    pub database: Database<StringMap>,
    pub application: Application,
    pub internal: Internal,
}


#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SiteSetting {
    pub name: String,
    pub description: String,
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